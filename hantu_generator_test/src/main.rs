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
    corpus.iter().for_each(|x| avg_tc_sz += x.len());
    assert!(corpus.len() > 0);
    avg_tc_sz /= corpus.len();
    println!("Average test case size: {avg_tc_sz} bytes");
    let token_dict = vec!["foobar".to_string(), "deadbeefcafebabe".to_string()];

    let mut mutation_engine = MutationEngine::new()
        .set_seed(0xdeadbeef)
        .set_corpus(corpus)
        .set_token_dict(token_dict)
        .set_printable(true)
        .set_max_mutation_size(5);

    let now = Instant::now();
    let mut i: usize = 0;
    loop {
        let tc = mutation_engine.mutate();
        i += 1;
        if i % THRESHOLD == 0 {
            println!(
                "Execs {:10} - {:8.1}/s - [{}]->{:10x?} - [{}]->'{:16?}' - [{}]-> {:?}",
                i,
                i as f64 / now.elapsed().as_secs_f64(),
                tc.get_idx(),
                tc.consume64(),
                tc.get_idx(),
                tc.consume_str(Some(16)),
                tc.get_idx(),
                tc.consume_vec(Some(16)),
            );
        }
    }
}
