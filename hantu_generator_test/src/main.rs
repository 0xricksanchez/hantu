use hantu::MutationEngine;
use std::sync::Arc;
use std::time::Instant;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const THRESHOLD: usize = 10000;

#[derive(Debug)]
pub enum Error {
    ReadingTestCase(std::io::Error),

    PathDoesNotExists(String),
}

fn load_corpus_from_disk<T: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>>(
    p: T,
) -> BTreeSet<Vec<u8>> {
    let mut corpus = BTreeSet::new();
    if Path::new(&p).is_dir() {
        let _ = fs::read_dir(&p)
            .map(|paths| {
                for path in paths {
                    insert_file_into_corpus(path.unwrap().path(), &mut corpus);
                }
            })
            .map_err(|_| Error::PathDoesNotExists("Failed to open corpus directory".to_string()));
    } else {
        insert_file_into_corpus(Path::new(&p).to_path_buf(), &mut corpus);
    }

    corpus.retain(|x| !x.is_empty());
    corpus
}

fn insert_file_into_corpus(f: PathBuf, corpus: &mut BTreeSet<Vec<u8>>) {
    if Path::new(&f).is_file() {
        let _ = fs::read(f)
            .map_err(Error::ReadingTestCase)
            .map(|c| corpus.insert(c));
    }
}

fn main() {
    println!("Hello, world!");
    let corpus: Arc<Vec<Vec<u8>>> =
        Arc::new(load_corpus_from_disk("corpus/").into_iter().collect());
    let mut avg_tc_sz = 0;
    let _ = corpus.iter().for_each(|x| avg_tc_sz += x.len());
    avg_tc_sz /= corpus.len();
    println!("Average test case size: {} bytes", avg_tc_sz);
    let mut token_dict = Vec::new();
    token_dict.push("foobar".to_string());
    token_dict.push("deadbeefcafebabe".to_string());

    let mut mutation_engine = MutationEngine::new(None, None, Some(token_dict), Some(corpus));
    let now = Instant::now();
    let mut i: usize = 0;
    loop {
        let _tc = mutation_engine.mutate();
        i += 1;
        if i % THRESHOLD == 0 {
            println!(
                "Execs {:10} - {:10.1}/s",
                i,
                i as f64 / now.elapsed().as_secs_f64(),
            );
        }
    }
}
