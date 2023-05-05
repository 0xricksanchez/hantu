# README

**TLDR:** Toy project that aims to build a usable fuzzer from the ground up
for educational purposes.

**Hantu** aims to provide a platform for learning and exploring various fuzzing
techniques and their inner workings. It's mainly a project for me to get a better
understanding of all low level concepts that are required in fuzzing. From building
a fuzzer from scratch, over operating system quirks, to performance optimizations
to in the end automatic bug hunting via effective fuzzing.

I aim at offering a wide range of features, some may not or never be production ready.
However, for benchmarking and educational purposes they'll likely remain in this
repository. You may now think that this oddly sounds like a bad rewrite of LibAFL
and you're not wrong.

## Why not LibAFL?

[LibAFL](https://github.com/AFLplusplus/LibAFL) is a genius project that
is ready to use in the real world to find bugs and its modular system
allows for a high customization. I like the philosophy of it.
However, as it basically builds on top of AFL(++) and years of experience
with ready made and established features it wasn't what I wanted.
I wanted a clean slate which I could model however I like.

While this may sounds like an insane task to build everything from scratch
it's all about the journey for me not about the quick _"hey look I found another
bug in binutils"_.

## Why Rust?

I'm a sucker for this language. While I'm myself still exploring all the capabilities
of Rust I found myself quite fond of it, so much that I decided to give
this a go :).

## Features

**Hantu** is in a very active WIP state that's still far from it being usable.
Currently, I implemented the following things to a varying degree:

- [ ] Mutator
  - [x] AFL/libfuzzer style byte and bit level mutations
    - [x] Shuffle bytes
    - [x] Erase bytes
    - [x] Insert bytes
    - [x] Swap neighbors with different widths
    - [x] Swap endianness with different widths
    - [x] Change a bit
    - [x] Change a byte
    - [x] Negate a byte
    - [x] Arithmetic operations on numbers
    - [x] Copy chunk
    - [x] Change an ASCII integer
    - [x] Change a Binary integer
    - [x] Cross-over
    - [x] Splice
    - [x] Truncate
    - [x] Append
    - [x] Add from magic constants
    - [x] Add from a TORC
  - [ ] Custom mutators
    - [x] Radamsa mutator based on [ni](https://github.com/aoh/ni)
    - [x] A grammar generator and a handful of grammars based on [F1](https://github.com/vrthra/F1)
      - You can check all the grammars in `src/libs/mutation_engine/src/custom_mutators/grammar_mutator/grammars`
- [ ] Custom Pseudo Random Number Generators
  - [x] I implemented a couple of different generators that you can find in `src/libs/prng/src/`:
    - [x] Lehmer64
    - [x] RomuDuoJr / RomuTrio
    - [x] ShiShua
    - [x] SplitMix64
    - [x] Xorshift
    - [x] XorShiro128**
    - [x] XorShuro256**
- [x] Magic constants
- [x] A simple `Command.run()` executor
- [x] A `TestCase` consumer interface

As this project is still in its very early stages expect things to break,
to have awful performance, missing tests/documentation or other horrors.

## Benchmarks

Some implementations, especially the PRNG, and grammar/ni mutator have some preliminary
benchmarks attached to them. Don't treat those as final or representative as things
shift around rather quickly.

## Future direction

The only clear goal of this is to learn as much as possible about the topic. So there's
not an expected feature set. However a minimal working fuzzer should have some guiding
mechanism. I'll tackle code-coverage or any other viable option in the foreseeable
future.

I'm also playing with the thought of going in the hypervisor direction with
this fuzzer. That said, nothing is set in stone and priorities will change
according to my time/interest.

## Contributions

As long as this is my toy project where I'm exploring things on my own and at
my own pace I won't accept feature requests of any kind. That said if some lost
soul ends up reading the code and finds issues in the current implementation,
whether it's a logic flaw or a performance loss I'd be happy to discuss those at
any point in time.
