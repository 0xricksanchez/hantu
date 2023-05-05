use core_affinity::CoreId;
use errors::{Error, Result};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{atomic::AtomicUsize, Arc},
    thread,
};
use test_case::TestCase;

use grammar_mutator::GrammarTemplate;
use mutation_engine::{CustomMutators, MutationEngine};
use prng::Generators;
use utils::{get_core_affinity, set_core_affinity};

#[derive(Debug, Clone, Default)]
pub struct FuzzerConfig {
    target: String,
    target_args: Vec<String>,
    corpus_dir: String,
    crash_dir: String,
    dict: Option<String>,
    pub max_iter: Option<usize>,
    batch_sz: usize,
    threads: Vec<CoreId>,
    generator: Generators,
    grammar: Option<String>,
    ni_mutator: bool,
    seed: usize,
    printable: bool,
    mutation_passes: usize,
}

impl FuzzerConfig {
    pub fn set_target(mut self, target: Vec<String>) -> Self {
        self.target = target[0].clone();
        assert!(Path::new(&self.target).exists(), "Target does not exist");
        assert!(Path::new(&self.target).is_file(), "Target is not a file");
        if target.len() > 1 {
            self.target_args = target[1..].to_vec();
        }
        self
    }

    fn ensure_dir(dir: &str) -> Result<String> {
        let p = Path::new(dir);
        if p.is_file() || p.is_symlink() {
            return Err(Error::NotADir(dir.to_string()));
        }
        if !p.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::CreatingDir(format!("Directory: {e}")))?;
        }
        if p.read_dir()?.next().is_some() {
            return Err(Error::NotEmpty(dir.to_string()));
        }
        Ok(dir.to_string())
    }

    pub fn set_corpus_dir(mut self, corpus_dir: &str) -> Self {
        let p = Path::new(corpus_dir);
        if p.is_file() || p.is_symlink() || !p.exists() {
            panic!("Corpus directory does not exist: {corpus_dir}");
        } else {
            self.corpus_dir = corpus_dir.to_string();
            self
        }
    }

    pub fn set_crash_dir(mut self, crash_dir: &str) -> Self {
        if let Err(e) = Self::ensure_dir(crash_dir) {
            panic!("Error setting crash directory: {e}");
        } else {
            self.crash_dir = crash_dir.to_string();
            self
        }
    }

    pub fn set_max_iter(mut self, max_iter: Option<usize>) -> Self {
        if max_iter.is_some() {
            self.max_iter = max_iter;
        }
        self
    }

    pub fn set_threads(mut self, threads: usize) -> Self {
        let ca = get_core_affinity(threads);
        if let Ok(threads) = ca {
            self.threads = threads;
        } else {
            panic!("Not enough cores available");
        }
        self
    }

    pub fn set_batch_sz(mut self, batch_sz: usize) -> Self {
        self.batch_sz = batch_sz;
        self
    }

    pub fn set_dict<T: AsRef<Path>>(mut self, dict: Option<T>) -> Self {
        if let Some(dict) = dict {
            assert!(dict.as_ref().exists(), "Dictionary file does not exist");
            self.dict = Some(dict.as_ref().to_str().unwrap().to_string());
        }
        self
    }

    pub fn set_seed(mut self, seed: usize) -> Self {
        self.seed = seed;
        self
    }

    pub fn set_generator(mut self, generator: Generators) -> Self {
        self.generator = generator;
        self
    }

    pub fn set_printable(mut self, printable: bool) -> Self {
        self.printable = printable;
        self
    }

    pub fn set_mutation_passes(mut self, mutation_passes: usize) -> Self {
        self.mutation_passes = mutation_passes;
        self
    }

    pub fn set_grammar(mut self, grammar: Option<String>) -> Self {
        if grammar.is_some() {
            self.grammar = grammar;
        }
        self
    }

    pub fn set_ni_mutator(mut self, ni_mutator: bool) -> Self {
        self.ni_mutator = ni_mutator;
        self
    }
}

#[derive(Default)]
pub struct FuzzerStats {
    iterations: AtomicUsize,
    crashes: AtomicUsize,
}

impl FuzzerStats {
    pub fn new() -> Self {
        Self {
            iterations: AtomicUsize::new(0),
            crashes: AtomicUsize::new(0),
        }
    }

    pub fn to_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn get_iterations(&self) -> usize {
        self.iterations.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn inc_iterations(&self) {
        self.iterations
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inc_iterations_by(&self, n: usize) {
        self.iterations
            .fetch_add(n, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inc_crashes(&self) {
        self.crashes
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_crashes(&self) -> usize {
        self.crashes.load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn load_corpus_from_disk<T: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>>(
    p: T,
) -> Arc<Vec<Vec<u8>>> {
    let mut corpus = BTreeSet::new();
    if Path::new(&p).is_dir() {
        let _ = std::fs::read_dir(&p).map(|dir| {
            dir.map(|entry| {
                entry.map(|e| {
                    let path = e.path();
                    if path.is_file() {
                        let _ = std::fs::read(path)
                            .map_err(Error::ReadingTestcase)
                            .map(|tc| corpus.insert(tc));
                    }
                })
            })
        });
    } else if Path::new(&p).is_file() {
        let _ = std::fs::read(p)
            .map_err(Error::ReadingTestcase)
            .map(|tc| corpus.insert(tc));
    };

    corpus.retain(|x| !x.is_empty());
    Arc::new(corpus.into_iter().collect())
}

fn get_mutation_engine(corp: &Arc<Vec<Vec<u8>>>, fuzz_config: &FuzzerConfig) -> MutationEngine {
    let mut me = MutationEngine::new()
        .set_corpus(corp.clone())
        .set_generator(&fuzz_config.generator)
        .set_generator_seed(fuzz_config.seed)
        .set_mutation_passes(fuzz_config.mutation_passes)
        .set_printable(fuzz_config.printable);
    if let Some(ref dict) = fuzz_config.dict {
        me = me.set_token_dict(dict);
    }
    let mut custom_mutators = Vec::new();
    if fuzz_config.ni_mutator {
        custom_mutators.push(CustomMutators::Ni);
    }

    if let Some(ref grammar) = fuzz_config.grammar {
        let g: GrammarTemplate = (*grammar).clone().into();
        custom_mutators.push(CustomMutators::GrammarGenerator(g));
    }

    if !custom_mutators.is_empty() {
        println!("[HANTU] Using custom mutators: {custom_mutators:?}");
        me = me.enable_custom_mutators(custom_mutators);
    }

    for _ in 0..128 {
        let tc_sz = me.prng.rand_range(0, 98304);
        let tc = me.prng.rand_byte_vec(tc_sz);
        me.add_to_corpus(&tc);
    }
    me
}

pub fn spawn_workers(fconfig: &FuzzerConfig, fstats: &Arc<FuzzerStats>) -> Result<()> {
    for (thr_id, &core_id) in fconfig.threads.iter().enumerate() {
        println!("[HANTU] Spawning a worker on core {core_id:?}");
        let mut fconfig = fconfig.clone();
        let fstats = fstats.clone();
        let _handle = thread::spawn(move || {
            set_core_affinity(&core_id).unwrap();
            worker(&mut fconfig, &fstats, thr_id).expect("Worker deployment successfully");
        });
    }
    Ok(())
}

fn fuzz_from_file<T: AsRef<Path>>(
    put: &str,
    put_args: &str,
    put_inp: T,
    tc: &mut TestCase,
) -> Result<Child> {
    fs::write(put_inp.as_ref(), tc.data.as_slice()).map_err(Error::WritingTestcase)?;
    let child = Command::new(put)
        .args(vec![put_args])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Error::SpawningTarget)?;
    Ok(child)
}

fn fuzz_from_stdin<T: AsRef<Path>>(
    put: &str,
    put_args: &str,
    _: T,
    tc: &mut TestCase,
) -> Result<Child> {
    let inp = unsafe { std::str::from_utf8_unchecked(tc.data.as_slice()) };
    let args = if put_args.is_empty() {
        vec![inp]
    } else {
        vec![put_args, inp]
    };
    let child = Command::new(put)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Error::SpawningTarget)?;
    Ok(child)
}

pub fn worker(fconfig: &mut FuzzerConfig, fstats: &Arc<FuzzerStats>, thr_id: usize) -> Result<()> {
    let corpus = load_corpus_from_disk(&fconfig.corpus_dir);
    let mut me = get_mutation_engine(&corpus, fconfig);
    let mut avg_tc_sz = 0;
    me.corpus.iter().for_each(|x| avg_tc_sz += x.len());
    avg_tc_sz /= me.corpus.len();
    println!("[HANTU] Average test case size: {avg_tc_sz} bytes");

    let inp_ff = format!(".tmp_inp_{thr_id}");

    let fuzz = if let Some(idx) = fconfig
        .target_args
        .iter()
        .position(|x| x == &"@@".to_string())
    {
        fconfig.target_args.remove(idx);
        fconfig.target_args.insert(idx, inp_ff.clone());
        fuzz_from_file::<&String>
    } else {
        fuzz_from_stdin::<&String>
    };

    me = me.set_random_test_case();
    let targs = fconfig.target_args.join(" ");

    loop {
        for _i in 0..fconfig.batch_sz {
            me.mutate();

            let mut child_proc = fuzz(&fconfig.target, &targs, &inp_ff, &mut me.test_case)?;
            match child_proc.wait().map_err(Error::WaitingForTarget) {
                Ok(status) => {
                    if status.success() {
                        //println!("exited with status: {exit_code}");
                        continue;
                    }
                    match status.code() {
                        Some(code) => {
                            if [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].contains(&code) {
                                println!("Exited with code: {code}");
                                fstats.inc_crashes();
                                let crash_file =
                                    format!(".crash_{thr_id}_{code}_{}", fstats.get_crashes());

                                fs::write(
                                    Path::new(&fconfig.crash_dir).join(crash_file),
                                    me.test_case.data.as_slice(),
                                )
                                .unwrap();
                            }
                        }
                        None => {
                            println!("Exited with signal");
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {e:?}");
                    let _ = child_proc.kill();
                }
            }
        }
        fstats.inc_iterations_by(fconfig.batch_sz);
    }
}
