#![feature(test)]

extern crate ni;
extern crate prng;
extern crate test;

use ni::{ni_area, ni_area_parallel, ni_area_parallel_hybrid};
use prng::xorshift::Xorshift64;
use prng::{Generator, Rng};
use std::sync::Arc;
use test::Bencher;

const ITERATIONS: usize = 1_000;
const CORPUS_SIZE: usize = 100;
const CORPUS_ENTRY_SIZE: [usize; 4] = [100, 1_000, 10_000, 100_000];

fn get_corpus(
    corpus_size: usize,
    corpus_entry_size: usize,
    prng: &mut Rng<Generator>,
) -> Arc<Vec<Vec<u8>>> {
    let mut corpus: Vec<Vec<u8>> = Vec::with_capacity(corpus_size);
    for _ in 0..corpus_size {
        let entry = prng.rand_byte_vec(corpus_entry_size);
        corpus.push(entry);
    }
    Arc::new(corpus)
}

fn bench_original_ni_area_size(b: &mut Bencher, size: usize) {
    let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0xdeadbeefcafebabe)));

    let corpus = get_corpus(CORPUS_SIZE, size, &mut prng);
    let data = &corpus[prng.rand() % corpus.len()];
    let mut out = Vec::new();

    b.iter(|| ni_area(data, ITERATIONS, &mut out, &mut prng, &corpus));
}

fn bench_parallel_ni_area_size(b: &mut Bencher, size: usize) {
    let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0xdeadbeefcafebabe)));

    let corpus = get_corpus(CORPUS_SIZE, size, &mut prng);
    let data = &corpus[prng.rand() % corpus.len()];
    let mut out = Vec::new();

    b.iter(|| ni_area_parallel(data, ITERATIONS, &mut out, &mut prng, &corpus));
}

fn bench_parallel_hybrid_ni_area_size(b: &mut Bencher, size: usize) {
    let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0xdeadbeefcafebabe)));

    let corpus = get_corpus(CORPUS_SIZE, size, &mut prng);
    let data = &corpus[prng.rand() % corpus.len()];
    let mut out = Vec::new();

    b.iter(|| ni_area_parallel_hybrid(data, ITERATIONS, &mut out, &mut prng, &corpus));
}

#[bench]
fn bench_original_ni_area_100(b: &mut Bencher) {
    bench_original_ni_area_size(b, CORPUS_ENTRY_SIZE[0]);
}

#[bench]
fn bench_original_ni_area_1k(b: &mut Bencher) {
    bench_original_ni_area_size(b, CORPUS_ENTRY_SIZE[1]);
}

#[bench]
fn bench_original_ni_area_10k(b: &mut Bencher) {
    bench_original_ni_area_size(b, CORPUS_ENTRY_SIZE[2]);
}

#[bench]
fn bench_original_ni_area_100k(b: &mut Bencher) {
    bench_original_ni_area_size(b, CORPUS_ENTRY_SIZE[3]);
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

#[bench]
fn bench_parallel_ni_area_100(b: &mut Bencher) {
    bench_parallel_ni_area_size(b, CORPUS_ENTRY_SIZE[0]);
}

#[bench]
fn bench_parallel_ni_area_1k(b: &mut Bencher) {
    bench_parallel_ni_area_size(b, CORPUS_ENTRY_SIZE[1]);
}

#[bench]
fn bench_parallel_ni_area_10k(b: &mut Bencher) {
    bench_parallel_ni_area_size(b, CORPUS_ENTRY_SIZE[2]);
}

#[bench]
fn bench_parallel_ni_area_100k(b: &mut Bencher) {
    bench_parallel_ni_area_size(b, CORPUS_ENTRY_SIZE[3]);
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=

#[bench]
fn bench_parallel_hybrid_ni_area_100(b: &mut Bencher) {
    bench_parallel_hybrid_ni_area_size(b, CORPUS_ENTRY_SIZE[0]);
}

#[bench]
fn bench_parallel_hybrid_ni_area_1k(b: &mut Bencher) {
    bench_parallel_hybrid_ni_area_size(b, CORPUS_ENTRY_SIZE[1]);
}

#[bench]
fn bench_parallel_hybrid_ni_area_10k(b: &mut Bencher) {
    bench_parallel_hybrid_ni_area_size(b, CORPUS_ENTRY_SIZE[2]);
}

#[bench]
fn bench_parallel_hybrid_ni_area_100k(b: &mut Bencher) {
    bench_parallel_hybrid_ni_area_size(b, CORPUS_ENTRY_SIZE[3]);
}
