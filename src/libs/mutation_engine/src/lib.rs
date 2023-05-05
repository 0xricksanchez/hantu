// Feature needs to stay here until issue #43244 is resolved: https://github.com/rust-lang/rust/issues/43244
#![feature(drain_filter)]
mod grammer_caller;

use errors::{Error, Result};
use magic::{MAGIC_16, MAGIC_32, MAGIC_64, MAGIC_8};
use num_traits::{
    AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub,
};

use prng::lehmer::Lehmer64;
use prng::romuduojr::RomuDuoJr;
use prng::romutrio::RomuTrio;
use prng::shishua::ShiShua;
use prng::splitmix::SplitMix64;
use prng::wyhash::Wyhash64;
use prng::xorshift::Xorshift64;
use prng::xorshiro128ss::XorShiro128ss;
use prng::xorshiro256ss::XorShiro256ss;
use prng::{Generator, Generators, Rng};
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::{path::Path, ptr, sync::Arc, usize};
use test_case::TestCase;

use grammar_mutator::{Grammar, GrammarTemplate, TokenIdentifier};
use grammer_caller::{GenerateFn, GrammarCaller};
use ni::ni_mutate;

#[derive(Debug, Clone)]
pub enum Mutators {
    Standard(StandardMutators),
    Custom(CustomMutators),
}

#[derive(Debug, Clone, Copy)]
pub enum StandardMutators {
    ShuffleBytes,
    EraseBytes,
    InsertBytes,
    SwapNeighbors,
    SwapEndianness,
    ChangeBit,
    ChangeByte,
    NegateByte,
    ArithmeticWidth,
    CopyPart,
    ChangeASCIIInteger,
    ChangeBinaryInteger,
    CrossOver,
    Splice,
    Truncate,
    Append,
    AddFromMagic,
    AddWordFromDict,
    AddWordFromTORC,
    Ni,
    GrammarGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomMutators {
    Ni,
    GrammarGenerator(GrammarTemplate),
}

pub struct MutationEngine {
    // List of available mutators to use
    mutators: Vec<Mutators>,
    // Function pointer to the grammar generator if set
    grammar_generator: GrammarCaller,
    // Start token for the grammar generator
    grammar_start: TokenIdentifier,
    // Maximum percentage of the test case to mutate
    max_mutation_factor: usize,
    // PRNG to use for mutations
    pub prng: Rng<Generator>,
    // Enforce ASCII printable mutations
    printable: bool,
    // User provided token dictionary
    user_token_dict: Vec<Vec<u8>>,
    // Mutation rounds per iteration
    mutation_passes: usize,
    // TORC dict filled dynamically during runtime
    torc_token_dict: Vec<Vec<u8>>,
    // The current test case to mutate
    pub test_case: TestCase,
    // Complete in-memory corpus
    pub corpus: Arc<Vec<Vec<u8>>>,
}

impl Default for MutationEngine {
    fn default() -> Self {
        let mutators = vec![
            Mutators::Standard(StandardMutators::ShuffleBytes),
            Mutators::Standard(StandardMutators::EraseBytes),
            Mutators::Standard(StandardMutators::InsertBytes),
            Mutators::Standard(StandardMutators::SwapNeighbors),
            Mutators::Standard(StandardMutators::SwapEndianness),
            Mutators::Standard(StandardMutators::ChangeBit),
            Mutators::Standard(StandardMutators::ChangeByte),
            Mutators::Standard(StandardMutators::NegateByte),
            Mutators::Standard(StandardMutators::ArithmeticWidth),
            Mutators::Standard(StandardMutators::CopyPart),
            Mutators::Standard(StandardMutators::ChangeASCIIInteger),
            Mutators::Standard(StandardMutators::ChangeBinaryInteger),
            Mutators::Standard(StandardMutators::CrossOver),
            Mutators::Standard(StandardMutators::Splice),
            Mutators::Standard(StandardMutators::Truncate),
            Mutators::Standard(StandardMutators::Append),
            Mutators::Standard(StandardMutators::AddFromMagic),
            Mutators::Standard(StandardMutators::AddWordFromTORC),
        ];

        let mut me = Self {
            mutators,
            grammar_generator: GrammarCaller::default(),
            grammar_start: TokenIdentifier(0),
            max_mutation_factor: 10,
            prng: Rng::new(Generator::Xorshift64(Xorshift64::new(0))),
            printable: false,
            user_token_dict: Vec::new(),
            mutation_passes: 1,
            torc_token_dict: Vec::new(),
            test_case: TestCase::default(),
            corpus: Arc::new(Vec::new()),
        };
        let initial_tc = me.prng.rand_byte_vec(128);
        me.add_to_corpus(&initial_tc);
        me
    }
}

impl MutationEngine {
    /// Create a new `MutationEngine` with default settings.
    /// The default settings are:
    ///
    /// * `mutators`: all available mutators
    /// * `max_mutation_factor`: 10
    /// * `prng`: Xorshift64
    /// * `printable`: false
    /// * `user_token_dict`: empty
    /// * `mutation_passes`: 1
    /// * `torc_token_dict`: empty
    /// * `test_case`: empty
    /// * `corpus`: empty
    ///
    /// # Returns
    ///
    /// A new `MutationEngine with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    /// let mutator = MutationEngine::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the PRNG with a given seed.
    /// This is useful for reproducible results. The default seed is 0.
    /// If you want to use a different seed, you should call this function before any mutations.
    /// If you change Generators, the seed will be reset to 0. You will need to call this function again.
    ///
    /// # Arguments
    ///
    /// * `seed` - The seed to use for the PRNG.
    ///
    /// # Returns
    ///
    /// A mutable reference to `Self` with the specified seed set.
    ///
    /// # Examples
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    /// let mutator = MutationEngine::new().set_generator_seed(1234);
    /// ```
    pub fn set_generator_seed(mut self, seed: usize) -> Self {
        self.prng.set_seed(seed);
        self
    }

    /// Sets the random number generator to a specified generator from the `Generators` enum.
    ///
    /// # Arguments
    ///
    /// * `prng` - A variant from the `Generators` enum representing the desired random number generator.
    ///
    /// # Returns
    ///
    /// A mutable reference to `Self` with the specified random number generator set.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::Generators;
    /// use mutation_engine::MutationEngine;
    /// let mut mutator = MutationEngine::new();
    ///
    /// mutator = mutator.set_generator(&Generators::Xorshift64);
    /// // `mutator` now uses the Xorshift64 generator.
    /// ```
    pub fn set_generator(mut self, prng: &Generators) -> Self {
        self.prng = match prng {
            Generators::Xorshift64 => self
                .prng
                .set_generator(Generator::Xorshift64(Xorshift64::new(0))),
            Generators::Splitmix64 => self
                .prng
                .set_generator(Generator::SplitMix64(SplitMix64::new(0))),
            Generators::Romuduojr => self
                .prng
                .set_generator(Generator::RomuDuoJr(RomuDuoJr::new(0))),
            Generators::Romutrio => self
                .prng
                .set_generator(Generator::RomuTrio(RomuTrio::new(0))),
            Generators::Xorshiro128ss => self
                .prng
                .set_generator(Generator::XorShiro128ss(XorShiro128ss::new(0))),
            Generators::Xorshiro256ss => self
                .prng
                .set_generator(Generator::XorShiro256ss(XorShiro256ss::new(0))),
            Generators::Lehmer64 => self
                .prng
                .set_generator(Generator::Lehmer64(Lehmer64::new(0))),
            Generators::Wyhash64 => self
                .prng
                .set_generator(Generator::Wyhash64(Wyhash64::new(0))),
            Generators::Shishua => self.prng.set_generator(Generator::ShiShua(ShiShua::new(0))),
        };
        self
    }

    /// Sets the corpus of the MutationEngine to the specified `Arc<Vec<Vec<u8>>>`.
    ///
    /// # Arguments
    ///
    /// * `corpus` - An `Arc<Vec<Vec<u8>>>` object containing the desired corpus.
    ///
    /// # Returns
    ///
    /// A mutable reference to `Self` with the specified corpus set.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::Arc;
    /// use mutation_engine::MutationEngine;
    /// let mut mutator = MutationEngine::new();
    ///
    /// let corpus = Arc::new(vec![vec![1u8, 2u8], vec![3u8, 4u8]]);
    /// mutator = mutator.set_corpus(corpus.clone());
    ///
    /// // Assert that the corpus has been set correctly.
    /// assert_eq!(mutator.corpus, corpus);
    /// ```
    pub fn set_corpus(mut self, corpus: Arc<Vec<Vec<u8>>>) -> Self {
        self.corpus = corpus;
        self
    }

    /// Adds a test case to the corpus.
    ///
    /// # Arguments
    ///
    /// * `test_case` - A `&Vec<u8>` representing the test case to be added to the corpus.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    /// use std::sync::Arc;
    ///
    /// let mut mutator = MutationEngine::new();
    /// let test_case = vec![5u8, 6u8];
    /// mutator.add_to_corpus(&test_case);
    ///
    /// // Assert that the test case has been added to the corpus.
    /// assert_eq!(mutator.corpus.last().unwrap(), &test_case);
    /// ```
    pub fn add_to_corpus(&mut self, test_case: &[u8]) {
        let corpus = Arc::make_mut(&mut self.corpus);
        corpus.push(test_case.to_vec());
    }

    /// Reads user tokens from a file and converts them to a `Vec<Vec<u8>>`.
    ///
    /// # Arguments
    ///
    /// * `tdict` - A path to the file containing user tokens separated by newlines.
    ///
    /// # Returns
    ///
    /// A `Vec<Vec<u8>>` containing the user tokens read from the file.
    fn user_tokens_to_vec<T: AsRef<Path>>(&mut self, tdict: T) -> Vec<Vec<u8>> {
        let mut file = File::open(tdict).expect("Failed to open dictionary file");
        let mut data = Vec::new();
        let mut buffer = [0; 8192];
        let mut last_line = Vec::new();
        loop {
            let n = file.read(&mut buffer).expect("Failed to read file");
            if n == 0 {
                break;
            }
            let buffer = &buffer[..n];
            for byte in buffer {
                last_line.push(*byte);
                if *byte == b'\n' {
                    data.push(last_line[..last_line.len() - 1].to_vec());
                    last_line.clear();
                }
            }
        }
        data
    }

    /// Sets the user token dictionary by loading tokens from the given file.
    ///
    /// # Arguments
    ///
    /// * `token_dict` - A path to the file containing user tokens separated by newlines.
    ///
    /// # Returns
    ///
    /// Self with the updated user token dictionary.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    /// use std::path::Path;
    ///
    /// let mut mutator = MutationEngine::new();
    /// let token_file_path = Path::new("dicts/test.dict");
    ///
    /// mutator = mutator.set_token_dict(token_file_path);
    /// ```
    pub fn set_token_dict<T: AsRef<Path>>(mut self, token_dict: T) -> Self {
        self.user_token_dict = self.user_tokens_to_vec(token_dict);
        println!(
            "[HANTU] Loaded {} tokens from user dictionary",
            self.user_token_dict.len()
        );
        self.mutators
            .push(Mutators::Standard(StandardMutators::AddWordFromDict));
        self
    }

    /// Enables custom mutators that are not as stable/fast as the others.
    /// This currently includes: `CustomMutator::Ni` and `CustomMutator::GrammarMutator`.
    /// The former closely resembles radamsa, and the latter generates a requested grammar
    ///
    ///
    /// # Returns
    ///
    /// Self with custom mutators enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    /// use mutation_engine::CustomMutators;
    ///
    /// let mut mutator = MutationEngine::new();
    /// mutator = mutator.enable_custom_mutators(vec![CustomMutators::Ni]);
    /// ```
    pub fn enable_custom_mutators(mut self, cm: Vec<CustomMutators>) -> Self {
        if cm.is_empty() {
            return self;
        }
        for custom_mutator in cm {
            match custom_mutator {
                CustomMutators::Ni => {
                    self.mutators.push(Mutators::Custom(CustomMutators::Ni));
                }
                CustomMutators::GrammarGenerator(gt) => {
                    let grammar: Grammar = Grammar::new(&gt).unwrap();

                    // Wrap the method call in a closure
                    self.grammar_start = grammar.start.unwrap();
                    let generate_fn: GenerateFn = Box::new(move |depth, id, prng, out| {
                        grammar.generate(depth, id, prng, out);
                    });

                    self.grammar_generator = GrammarCaller { generate_fn };

                    self.mutators
                        .push(Mutators::Custom(CustomMutators::GrammarGenerator(gt)));
                }
            }
        }

        self
    }

    /// Clears the list of mutators.
    pub fn clear_mutators(&mut self) {
        self.mutators.clear();
    }

    /// Sets whether the mutated data should be printable ASCII characters.
    ///
    /// # Arguments
    ///
    /// * `printable` - If true, the mutated data will be printable ASCII characters.
    ///
    /// # Returns
    ///
    /// Self with the updated printable property.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// mutator = mutator.set_printable(true);
    /// ```
    pub fn set_printable(mut self, printable: bool) -> Self {
        self.printable = printable;
        self
    }

    /// Sets the maximum mutation size factor to use when mutating a test case in percentage
    /// values. This is currently used in only two mutators `Mutator::erase_bytes` and `Mutator::insert_bytes`.
    ///
    /// # Arguments
    ///
    /// * `num_factor` - The maximum mutation size factor to set (must be between 1 and 99, inclusive).
    ///
    /// # Returns
    ///
    /// Self with the updated maximum mutation size factor.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// mutator = mutator.set_max_mutation_size(25);
    /// ```
    pub fn set_max_mutation_size(mut self, num_factor: usize) -> Self {
        if num_factor == 0 || num_factor >= 100 {
            self.max_mutation_factor = 10;
        } else {
            self.max_mutation_factor = num_factor;
        }
        self
    }

    /// Sets the number of mutation passes per mutation. The default is 1 to avoid
    /// slowing down the fuzzer too much. Higher values can be used to increase
    /// the mutation rate of the test cases in each iteration.
    ///
    /// # Arguments
    ///
    /// * `rounds` - The number of mutation passes to set.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// mutator = mutator.set_mutation_passes(5);
    /// ```
    pub fn set_mutation_passes(mut self, rounds: usize) -> Self {
        self.mutation_passes = rounds;
        self
    }

    /// Set a new test case from the corpus or generate a new byte array one if the corpus is empty.
    fn set_new_test_case(&mut self) {
        let corpus_len = self.corpus.len();
        assert!(corpus_len > 0, "Corpus is empty");
        self.test_case.data.clear();
        self.test_case.data_ptr = 0;

        let idx = self.prng.rand_range(0, corpus_len);
        let chosen = &self.corpus[idx];

        self.test_case.data.extend_from_slice(chosen);
        self.test_case.size = chosen.len();
    }

    /// Sets the test case with the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to a `Vec<u8>` containing the test case data.
    ///
    /// # Returns
    ///
    /// Mutable reference to Self with the updated test case.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// let test_case_data = vec![1u8, 2, 3, 4, 5];
    ///
    /// mutator.set_test_case(&test_case_data);
    /// assert!(mutator.test_case.data == test_case_data);
    /// ```
    pub fn set_test_case(&mut self, data: &Vec<u8>) -> &mut Self {
        self.test_case = TestCase::new(data);
        self
    }

    /// Sets a random entry from the corpus as the test case.
    ///
    /// # Returns
    ///
    /// Self with the updated test case.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// let mut corpus = vec![vec![1u8, 2, 3, 4, 5], vec![6, 7, 8, 9, 10]];
    /// mutator = mutator.set_corpus(corpus.into());
    /// mutator = mutator.set_random_test_case();
    /// assert!(mutator.test_case.data == vec![6u8, 7, 8, 9, 10] || mutator.test_case.data == vec![1u8, 2, 3, 4, 5]);
    /// ```
    pub fn set_random_test_case(mut self) -> Self {
        let tc = self.get_random_corpus_entry();
        self.set_test_case(&tc);
        self
    }

    /// This function will return a random entry from the corpus as a vector of bytes.
    /// If the corpus is empty it will return a random vector of bytes with a size of 128.
    fn get_random_corpus_entry(&mut self) -> Vec<u8> {
        if let Some(tc) = self.corpus.get(self.prng.rand_range(0, self.corpus.len())) {
            tc.clone()
        } else {
            self.prng.rand_byte_vec(128)
        }
    }

    /// This is a helper function that will ensure that a byte is printable
    fn ensure_printable(&mut self) -> u8 {
        let b = self.prng.rand_byte();
        if self.printable {
            b.wrapping_sub(32) % 95 + 32
        } else {
            b
        }
    }

    /// Mutates the current test case based on the selected mutation strategy and mutation passes.
    ///
    /// # Returns
    ///
    /// Mutable reference to the mutated `TestCase`.
    ///
    /// # Example
    ///
    /// ```
    /// use mutation_engine::MutationEngine;
    ///
    /// let mut mutator = MutationEngine::new();
    /// let test_case_data = vec![1u8, 2, 3, 4, 5];
    ///
    /// mutator.set_test_case(&test_case_data);
    /// let mutated_test_case = mutator.mutate();
    /// assert!(mutated_test_case.data != test_case_data);
    /// ```
    pub fn mutate(&mut self) -> &mut TestCase {
        self.set_new_test_case();
        for _ in 0..self.mutation_passes {
            let _ = match self.prng.pick(&self.mutators) {
                Mutators::Standard(StandardMutators::ShuffleBytes) => self.shuffle_bytes(),
                Mutators::Standard(StandardMutators::EraseBytes) => self.erase_bytes(),
                Mutators::Standard(StandardMutators::InsertBytes) => self.insert_bytes(),
                Mutators::Standard(StandardMutators::SwapNeighbors) => self.swap_neighbors(),
                Mutators::Standard(StandardMutators::SwapEndianness) => self.swap_endianness(),
                Mutators::Standard(StandardMutators::ChangeBit) => self.change_bit(),
                Mutators::Standard(StandardMutators::ChangeByte) => self.change_byte(),
                Mutators::Standard(StandardMutators::ArithmeticWidth) => self.arithmetic_width(),
                Mutators::Standard(StandardMutators::NegateByte) => self.negate_byte(),
                Mutators::Standard(StandardMutators::CopyPart) => self.copy_part(),
                Mutators::Standard(StandardMutators::ChangeASCIIInteger) => {
                    self.change_ascii_integer()
                }
                Mutators::Standard(StandardMutators::ChangeBinaryInteger) => {
                    self.change_binary_integer()
                }
                Mutators::Standard(StandardMutators::CrossOver) => self.cross_over(),
                Mutators::Standard(StandardMutators::Splice) => self.splice(),
                Mutators::Standard(StandardMutators::Truncate) => self.truncate(),
                Mutators::Standard(StandardMutators::Append) => self.append(),
                Mutators::Standard(StandardMutators::AddFromMagic) => self.add_from_magic(),
                Mutators::Standard(StandardMutators::AddWordFromDict) => self.add_word_from_dict(),
                Mutators::Standard(StandardMutators::AddWordFromTORC) => self.add_word_from_torc(),
                Mutators::Custom(CustomMutators::Ni) => self.ni(),
                Mutators::Custom(CustomMutators::GrammarGenerator(_)) => self.grammar_gen(),
                _ => unreachable!(),
            };
        }
        &mut self.test_case
    }

    /// Mutator that generates a grammar output based on the grammar requested
    fn grammar_gen(&mut self) -> Result<()> {
        let mut out: Vec<u8> = Vec::new();
        self.grammar_generator
            .call_generate(0, self.grammar_start, &mut self.prng, &mut out);
        self.set_test_case(&out);
        Ok(())
    }

    /// Mutator based on <https://github.com/aoh/ni>
    fn ni(&mut self) -> Result<()> {
        let res = ni_mutate(
            &self.test_case.data,
            self.test_case.size,
            &mut self.prng,
            &self.corpus,
        );
        self.set_test_case(&res.unwrap());
        Ok(())
    }

    /// Shuffles a slice of `self.test_case.data` with a random length between 1 and 8 (inclusive),
    /// starting at a random index. If `self.test_case.size` is less than or equal to 2, returns an
    /// error indicating that there's nothing to shuffle.
    fn shuffle_bytes(&mut self) -> Result<()> {
        if self.test_case.size < 2 {
            return Err(Error::new("Nothing to shuffle"));
        }
        let shuffle_amount = self
            .prng
            .rand_range(1, std::cmp::min(self.test_case.size, 8))
            + 1;
        let shuffle_start = self
            .prng
            .rand_range(0, self.test_case.size - shuffle_amount);
        self.prng
            .shuffle(&mut self.test_case.data[shuffle_start..shuffle_start + shuffle_amount]);
        Ok(())
    }

    /// Mutator that erases a random amount ([1; min(100, `test_case.size` * 0.1)]) of bytes from the test case
    fn erase_bytes(&mut self) -> Result<()> {
        if self.test_case.size == 0 {
            return Err(Error::new("Nothing to delete"));
        }

        // Have a 50% chance to only remove one arbitrary byte
        if self.prng.bool() {
            let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
            self.test_case.data.remove(idx);
            self.test_case.size -= 1;
        } else {
            // Delete at most 10% of the data but no more than 100 for large inputs as erasing is expensive
            // and we don't want to have this as a bottleneck
            let max_factor = if self.test_case.size < 20 {
                self.test_case.size
            } else {
                std::cmp::min(100, self.test_case.size / self.max_mutation_factor)
            };

            for _ in 0..max_factor {
                let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
                self.test_case.data.remove(idx);
                self.test_case.size -= 1;
            }
        }

        Ok(())
    }

    /// Mutator that inserts a random amount ([1; min(100, `test_case.size` * 0.1)]) of bytes into the test case
    fn insert_bytes(&mut self) -> Result<()> {
        let to_insert = self.ensure_printable();
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
        // 50% chance to only insert one byte
        if self.prng.bool() {
            let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
            self.test_case.data.insert(idx, to_insert);
            self.test_case.size += 1;
        } else {
            let max_factor = if self.test_case.size < 20 {
                self.test_case.size
            } else {
                std::cmp::min(100, self.test_case.size / self.max_mutation_factor)
            };
            self.test_case
                .data
                .splice(idx..idx, std::iter::repeat(to_insert).take(max_factor));
            self.test_case.size += max_factor;
        }
        Ok(())
    }

    /// Swaps two (q|d|w) word, or byte neighbors in the test case
    fn swap_neighbors(&mut self) -> Result<()> {
        let fun: fn(&mut Vec<u8>, usize, &mut Rng<Generator>) -> Result<()> =
            match self.prng.rand_range(0, 4) {
                0 => swap_neighbors_width::<u8>,
                1 => swap_neighbors_width::<u16>,
                2 => swap_neighbors_width::<u32>,
                3 => swap_neighbors_width::<u64>,
                _ => unreachable!(),
            };

        fun_caller(
            fun,
            &mut self.test_case.data,
            self.test_case.size,
            &mut self.prng,
        )
    }

    /// Swaps the endianess of a random amount ([2,4, 8]) of bytes in the test case
    fn swap_endianness(&mut self) -> Result<()> {
        let mut width = self.prng.pick([2, 4, 8]) as usize;
        if self.test_case.size < width {
            return Err(Error::new("Mutation size > test case"));
        }
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, Some(1));
        width = usize::min(width, self.test_case.size - idx);
        let slice = &mut self.test_case.data[idx..idx + width];
        let ptr = slice.as_mut_ptr();
        for i in 0..(width >> 1) {
            unsafe {
                std::ptr::swap(ptr.add(i), ptr.add(width - i - 1));
            }
        }
        Ok(())
    }

    /// Mutator that changes a random bit in the test case
    fn change_bit(&mut self) -> Result<()> {
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
        let bit = self.prng.rand_range(0, 8);
        self.test_case.data[idx] ^= 1 << bit;
        Ok(())
    }

    /// Mutator that changes a random byte in the test case by either replacing it with a random byte or
    /// XOR'ing it with a random byte
    fn change_byte(&mut self) -> Result<()> {
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
        let byte = &mut self.test_case.data[idx];
        let r = self.prng.rand_byte();
        if self.prng.bool() {
            if r == *byte {
                *byte = r + 1;
            } else {
                *byte = r;
            }
        } else if r == 0 {
            *byte ^= r + 1;
        } else {
            *byte ^= r;
        }
        Ok(())
    }

    /// Mutator that negates a random byte in the test case
    fn negate_byte(&mut self) -> Result<()> {
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, None);
        let byte = &mut self.test_case.data[idx];
        *byte = !*byte;
        Ok(())
    }

    /// Mutator that treats [1,2,4,8] bytes in the test case as an integer and performs an arithmetic operation on it
    fn arithmetic_width(&mut self) -> Result<()> {
        let fun: fn(&mut Vec<u8>, usize, &mut Rng<Generator>) -> Result<()> =
            match self.prng.rand_range(0, 4) {
                0 => arithmetic::<u8>,
                1 => arithmetic::<u16>,
                2 => arithmetic::<u32>,
                3 => arithmetic::<u64>,
                _ => unreachable!(),
            };
        fun_caller(
            fun,
            &mut self.test_case.data,
            self.test_case.size,
            &mut self.prng,
        )
    }

    /// Mutator that changes a random byte in the test case that is within ASCII range
    fn change_ascii_integer(&mut self) -> Result<()> {
        let skip_past = self.prng.rand_range(0, self.test_case.size);

        let data = &self.test_case.data[skip_past..];
        let ascii_digits = data
            .windows(data.len())
            .position(|window| window.iter().all(u8::is_ascii_digit))
            .map(|i| (i + skip_past, i + data.len() + skip_past));
        let (start, end) = ascii_digits.unwrap_or((self.test_case.size, self.test_case.size));

        // No range of ASCII digits found. Simply flip the first byte and return;
        if start == self.test_case.size && end == self.test_case.size {
            self.test_case.data[0] = !self.test_case.data[0];
            return Ok(());
        }
        let mut val: u8 = self.test_case.data[start..end]
            .iter()
            .enumerate()
            .fold(0u8, |acc, (i, &ch)| {
                acc + (ch - b'0') * (10_u8.wrapping_mul(i as u8))
            });
        match self.prng.rand_range(0, 5) {
            0 => val = val.wrapping_add(1),
            1 => val = val.wrapping_sub(1),
            2 => val /= 2,
            3 => val = val.wrapping_mul(2),
            4 => val = (self.prng.rand_range(0, val as usize * val as usize)) as u8,
            _ => unreachable!(),
        }
        if val > 9 {
            val = 9;
        }
        self.test_case.data[start..end]
            .iter_mut()
            .enumerate()
            .rev()
            .for_each(|(_, ch)| *ch = val + b'0');

        Ok(())
    }

    /// Changes a random byte in the test case that is not within ASCII range
    fn change_binary_integer(&mut self) -> Result<()> {
        let mut val: usize;
        let bin_size: usize = *self.prng.pick(&[1, 2, 4, 8]) as usize;
        if self.test_case.size < bin_size {
            return Err(Error::new("Mutation size > test case"));
        }
        let off = self.prng.rand_range(0, self.test_case.size - bin_size + 1);
        let add = (self.prng.rand_range(0, 21) as isize - 10).max(0) as usize;
        val =
            if off < 64 && self.prng.bool_chance(4) {
                self.test_case.size
            } else {
                match bin_size {
                    1 => u8::from_be_bytes(
                        self.test_case.data[off..off + bin_size].try_into().unwrap(),
                    ) as usize,
                    2 => u16::from_be_bytes(
                        self.test_case.data[off..off + bin_size].try_into().unwrap(),
                    ) as usize,
                    4 => u32::from_be_bytes(
                        self.test_case.data[off..off + bin_size].try_into().unwrap(),
                    ) as usize,
                    8 => u64::from_be_bytes(
                        self.test_case.data[off..off + bin_size].try_into().unwrap(),
                    ) as usize,
                    _ => unreachable!(),
                }
            };
        if self.prng.bool() {
            val = match bin_size {
                1 => u8::swap_bytes(val as u8).wrapping_add(add as u8) as usize,
                2 => u16::swap_bytes(val as u16).wrapping_add(add as u16) as usize,
                4 => u32::swap_bytes(val as u32).wrapping_add(add as u32) as usize,
                8 => u64::swap_bytes(val as u64).wrapping_add(add as u64) as usize,
                _ => unreachable!(),
            };
        } else {
            val = val.wrapping_add(add);
        };

        if add == 0 || self.prng.bool() {
            if add == val {
                val = self.prng.rand_byte() as usize;
            }
            val = val.wrapping_neg();
        }
        match bin_size {
            1 => {
                self.test_case.data[off..off + bin_size]
                    .copy_from_slice(&((val & 0xFF) as u8).to_be_bytes());
            }
            2 => {
                self.test_case.data[off..off + bin_size]
                    .copy_from_slice(&((val & 0xFFFF) as u16).to_be_bytes());
            }
            4 => {
                self.test_case.data[off..off + bin_size]
                    .copy_from_slice(&((val & 0xFFFF_FFFF) as u32).to_be_bytes());
            }
            8 => {
                self.test_case.data[off..off + bin_size]
                    .copy_from_slice(&(val as u64).to_be_bytes());
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    /// Mutator that either copies a random part of another test case to a random location of the current
    /// test case overwriting existing data, or inserts a random part of another test case into the
    /// current test case at a random location without overwriting existing data.
    fn copy_part(&mut self) -> Result<()> {
        let rand_test_case = self.get_random_corpus_entry();
        assert!(!rand_test_case.is_empty(), "Copy part candidate is empty");
        if self.prng.bool() {
            copy_part_of(&rand_test_case, &mut self.test_case, &mut self.prng)
        } else {
            let max_size = self.test_case.size + self.prng.rand_range(1, self.test_case.size);
            insert_part_of(
                &rand_test_case,
                &mut self.test_case,
                &mut self.prng,
                max_size,
            )
        }
    }

    /// Mutator that combines two random test cases using a cross over operation.
    fn cross_over(&mut self) -> Result<()> {
        let mut data2 = self.get_random_corpus_entry();
        let size2 = data2.len();
        assert!(size2 > 0, "Cross over candidate is empty");

        let data1 = &mut self.test_case.data;
        let size1 = self.test_case.size;
        let max_out_size = self.prng.rand() % (data1.len() + data2.len()) + 1;
        let mut out = vec![0u8; max_out_size];
        let mut out_pos = 0;
        let mut pos1 = 0;
        let mut pos2 = 0;
        let mut currently_using_first_data = true;
        while out_pos < max_out_size && (pos1 < size1 || pos2 < size2) {
            let out_size_left = max_out_size - out_pos;
            let (in_pos, in_size, data) = if currently_using_first_data {
                (&mut pos1, size1, data1.as_mut_slice())
            } else {
                (&mut pos2, size2, &mut *data2)
            };
            if *in_pos < in_size {
                let in_size_left = in_size - *in_pos;
                let max_extra_size = std::cmp::min(out_size_left, in_size_left);
                let extra_size = self.prng.rand() % (max_extra_size + 1);
                if *in_pos + extra_size <= data.len() && out_pos < max_out_size {
                    out[out_pos..(out_pos + extra_size)]
                        .copy_from_slice(&data[*in_pos..*in_pos + extra_size]);
                    out_pos += extra_size;
                    *in_pos += extra_size;
                }
            }
            currently_using_first_data = !currently_using_first_data;
        }
        self.test_case.size = max_out_size;
        self.test_case.data = out;
        Ok(())
    }

    /// Mutator that splices a random part of another test case into the current test case at
    /// a random location.
    fn splice(&mut self) -> Result<()> {
        assert!(self.corpus.len() > 0, "corpus is empty");
        // `Clone` is not implemented for `Arc` so we get our reference to a test case by index.
        let splice_tc = self.prng.pick(self.corpus.as_slice());
        let split_idx = self.prng.rand_range(0, self.test_case.size);
        let splice_idx = self.prng.rand_range(0, splice_tc.len());
        // This is way faster than using the actual built-in splice function.
        let mut new_data = Vec::with_capacity(split_idx + splice_tc.len() - splice_idx);
        new_data.extend_from_slice(&self.test_case.data[..split_idx]);
        new_data.extend_from_slice(&splice_tc[splice_idx..]);
        self.test_case.size = new_data.len();
        self.test_case.data = new_data;
        // self.test_case.data.splice(split_idx.., splice_tc[..splice_idx].iter().cloned());
        Ok(())
    }

    /// Mutator that removes a randomly sized chunk of the current test case.
    fn truncate(&mut self) -> Result<()> {
        let trunc_fac = (self.prng.rand_range(0, 50) + 1) as f64 * 0.01;
        self.test_case.size = (self.test_case.size as f64 * (1.0 - trunc_fac)) as usize;
        self.test_case.data.truncate(self.test_case.size);
        Ok(())
    }

    /// Mutator that appends a random sized chunk of the current test case to itself.
    fn append(&mut self) -> Result<()> {
        let from = self
            .prng
            .rand_range(0, self.test_case.size - self.mutation_passes);
        let to = from + self.mutation_passes;
        let mut to_append = self.test_case.data[from..to].to_vec();
        self.test_case.data.append(&mut to_append);
        self.test_case.size += self.mutation_passes;
        Ok(())
    }

    /// Mutator that inserts a constant value from the magic set into the current test case.
    fn add_from_magic(&mut self) -> Result<()> {
        // Roll a 4 sided dice to decide which val to read from
        let dice_roll = self.prng.rand_range(0, 4);
        let val: usize;
        let val_size: usize;
        match dice_roll {
            0 => {
                val = self.prng.pick(MAGIC_8) as usize;
                val_size = std::mem::size_of::<u8>();
            }
            1 => {
                val = self.prng.pick(MAGIC_16) as usize;
                val_size = std::mem::size_of::<u16>();
            }
            2 => {
                val = self.prng.pick(MAGIC_32) as usize;
                val_size = std::mem::size_of::<u32>();
            }
            3 => {
                val = self.prng.pick(MAGIC_64) as usize;
                val_size = std::mem::size_of::<u64>();
            }
            _ => unreachable!(),
        }

        // Roll a random index into data
        if val_size > self.test_case.size {
            return Err(Error::new("Mutation size > test case"));
        }
        let idx = get_random_index(&mut self.test_case.data, &mut self.prng, Some(val_size));
        if idx + val_size >= self.test_case.size {
            return Err(Error::new("Mutation size > test case"));
        }

        if val == 0 {
            // Write the val to data
            for i in 0..val_size {
                self.test_case.data[idx + i] = 0;
            }
        } else {
            // Check if val has any unset upper bytes
            let mut v = val;
            let mut unset_bytes = 0;
            while v > 0 {
                v >>= 8;
                unset_bytes += 1;
            }
            let start = idx + val_size - unset_bytes;
            let end = idx + val_size;
            for i in start..end {
                self.test_case.data[i] = (val >> (8 * (i - start))) as u8;
            }
        }
        Ok(())
    }

    /// Mutator that inserts a random value from the user token dictionary into the current test case.
    fn add_word_from_dict(&mut self) -> Result<()> {
        add_from_dict(
            &self.user_token_dict,
            &mut self.test_case.data,
            &mut self.prng,
        )
    }

    /// Mutator that inserts a random value from the TORC token dictionary into the current test case.
    fn add_word_from_torc(&mut self) -> Result<()> {
        if self.torc_token_dict.is_empty() {
            return Err(Error::new("TORC token dict is empty"));
        };
        add_from_dict(
            &self.torc_token_dict,
            &mut self.test_case.data,
            &mut self.prng,
        )
    }
}

/// Returns a random index into data. If `exclude_off` is not None, the returned index will be at least
/// `exclude_off` bytes away from the end of data.
fn get_random_index(
    data: &mut Vec<u8>,
    prng: &mut Rng<Generator>,
    exclude_off: Option<usize>,
) -> usize {
    assert!(!data.is_empty(), "Cannot get random index from empty data");
    prng.rand_exp(0, data.len() - exclude_off.map_or(0, |x| x))
}

/// Adds a random value from dict to data.
fn add_from_dict(dict: &[Vec<u8>], data: &mut Vec<u8>, prng: &mut Rng<Generator>) -> Result<()> {
    assert!(!dict.is_empty(), "Cannot add from empty dict");
    let mut val = prng.pick(dict).clone();
    let val_size = val.len();
    if val_size > data.len() {
        return Err(Error::new("Dictionary token larger than test case"));
    }
    let to = prng.rand_range(0, data.len() - val_size);
    if val_size == 1 {
        data[to] = val[0];
        return Ok(());
    }
    if prng.bool() {
        val.reverse();
    };
    (0..val_size).for_each(|i| {
        data[to + i] = val[i];
    });
    Ok(())
}

/// Unlike splicing this function will not modify the size of the test case. It will instead
/// copy a random sized chunk of another test case into the current test case within its bounds.
fn copy_part_of(from: &Vec<u8>, to: &mut TestCase, prng: &mut Rng<Generator>) -> Result<()> {
    let mut to_idx = prng.rand_range(0, to.size);
    let mut copy_size = std::cmp::min(prng.rand_range(1, to.size - to_idx + 1), from.len());
    let mut from_idx = prng.rand_range(0, from.len() - copy_size + 1);
    if copy_size == 1 {
        let b = from[from_idx];
        to.data[to_idx] ^= if b == 0 { b + 1 } else { b };
        return Ok(());
    }
    if from_idx == to_idx {
        if from_idx > 0 {
            from_idx -= 1;
        } else if to_idx > 0 {
            to_idx -= 1;
        } else if from_idx + 1 + copy_size < from.len() {
            from_idx += 1;
        } else {
            from_idx += 1;
            copy_size -= 1;
        }
    }
    let copy_slice = &from[from_idx..from_idx + copy_size];
    to.data[to_idx..to_idx + copy_size].clone_from_slice(copy_slice);
    Ok(())
}

/// This function modifies the test case by inserting a portion from another corpus entry
/// at a random position, without overwriting any data.
fn insert_part_of(
    from: &[u8],
    to: &mut TestCase,
    prng: &mut Rng<Generator>,
    max_size: usize,
) -> Result<()> {
    let available_space = max_size - to.size;
    let max_copy_size = std::cmp::min(available_space, from.len());
    assert!(max_copy_size > 0, "Insertion size is 0");

    let copy_size = prng.rand_range(1, max_copy_size + 1);
    let from_idx = prng.rand_range(0, from.len() - copy_size + 1);
    let to_idx = if to.data.is_empty() {
        0
    } else {
        prng.rand_range(0, to.size)
    };

    /*
    to.data.resize(to.size + copy_size, 0);

    // Rotate the slice to the right
    to.data[to_idx..].rotate_right(copy_size);

    // Copy the data from the other corpus entry into the open spot
    to.data[to_idx..to_idx + copy_size].copy_from_slice(&from[from_idx..from_idx + copy_size]);

    to.size = to.data.len();
    Ok(())
    */

    // Allocate new memory for the data.
    // This seems to be faster than relying on the `resize` and `rotate_right` functions
    // that are implemented on `Vec`. Experiments show a 6% speedup.
    let new_size = to.size + copy_size;
    let mut new_data: Vec<u8> = vec![0u8; new_size];
    unsafe {
        new_data.set_len(new_size);
    }

    // Copy the data before the insertion point
    new_data[..to_idx].copy_from_slice(&to.data[..to_idx]);

    // Copy the data from `from`
    new_data[to_idx..to_idx + copy_size].copy_from_slice(&from[from_idx..from_idx + copy_size]);

    // Copy the data after the insertion point
    new_data[to_idx + copy_size..].copy_from_slice(&to.data[to_idx..]);

    to.data = new_data;
    to.size = new_size;
    Ok(())
}

fn arithmetic<T>(data: &mut Vec<u8>, data_size: usize, prng: &mut Rng<Generator>) -> Result<()>
where
    T: num_traits::PrimInt
        + num_traits::Unsigned
        + WrappingAdd
        + WrappingSub
        + WrappingNeg
        + WrappingMul
        + WrappingShl
        + WrappingShr
        + std::ops::BitOrAssign
        + num::cast::AsPrimitive<u8>
        + std::convert::From<u8>,
    u8: AsPrimitive<T>,
{
    let bytes = std::mem::size_of::<T>();
    if data_size < bytes {
        return Err(Error::new("Mutation size > test case"));
    }
    let idx = get_random_index(data, prng, Some(bytes));
    let mut val: T = 0.into();
    for i in 0..bytes {
        val |= <u8 as AsPrimitive<T>>::as_(data[idx + i]) << (8 * (bytes - i - 1));
    }
    let op = prng.rand_range(0, 6);
    val = match op {
        0 => val.wrapping_sub(&1.into()),
        1 => val.wrapping_add(&1.into()),
        2 => val.wrapping_mul(&2.into()),
        3 => val.wrapping_neg(),
        4 => val.wrapping_shl(2),
        5 => val.wrapping_shr(2),
        _ => unreachable!(),
    };
    for i in 0..bytes {
        let a = 8 * (bytes - i - 1);
        let b = val >> a;
        data[idx + i] = b.as_();
    }
    Ok(())
}

fn swap_neighbors_width<T>(
    data: &mut Vec<u8>,
    data_size: usize,
    prng: &mut Rng<Generator>,
) -> Result<()>
where
    T: num_traits::PrimInt + num_traits::Unsigned,
{
    let bytes = std::mem::size_of::<T>();
    if data_size <= bytes {
        return Err(Error::new("Mutation size > test case"));
    }
    let idx = get_random_index(data, prng, Some(data_size - 1 - bytes));
    // Check if we can swap with the right neighbor
    if idx + 2 * bytes < data_size {
        for i in 0..bytes {
            unsafe { ptr::swap(&mut data[idx + i], &mut data[idx + bytes + i]) }
        }
    } else if (idx as i64 - bytes as i64) >= 0 && idx + bytes < data_size {
        // Otherwise swap with the left neighbors
        for i in 0..bytes {
            unsafe {
                ptr::swap(&mut data[idx - bytes + i], &mut data[idx + i]);
            }
        }
    } else {
        // If swapping with the left or the right is not possible,
        // calculate the maximum number of bytes that can be swapped
        let max_bytes = (data_size - idx).min(idx).min(bytes);
        if max_bytes < 2 {
            // Otherwise just negate the byte
            data[max_bytes] = !data[max_bytes];
            return Ok(());
        }
        let half_bytes = max_bytes / 2;
        if (idx as i64 - half_bytes as i64) >= 0 {
            // swap with the left
            for i in 0..half_bytes {
                unsafe {
                    ptr::swap(&mut data[idx - i], &mut data[idx + half_bytes - i]);
                }
            }
        } else {
            // swap with the right
            for i in 0..half_bytes {
                unsafe {
                    ptr::swap(&mut data[idx + i], &mut data[data_size - i - 1]);
                }
            }
        }
    }
    Ok(())
}

/// A function that calls a function pointer
fn fun_caller(
    func: fn(&mut Vec<u8>, usize, &mut Rng<Generator>) -> Result<()>,
    data: &mut Vec<u8>,
    data_size: usize,
    prng: &mut Rng<Generator>,
) -> Result<()> {
    func(data, data_size, prng)
}

#[cfg(test)]
mod tests {
    use super::*;

    enum TestCondition {
        DataInequality,
        SizeInequality,
        GeneralErrorChecker,
    }

    fn corpus() -> Arc<Vec<Vec<u8>>> {
        let corpus: Arc<Vec<Vec<u8>>> = Arc::new(vec![
            "ThisIsSomeTest".as_bytes().to_vec(),
            "YetAnotherSimpleInput".as_bytes().to_vec(),
            [0xff].to_vec(),
        ]);
        corpus
    }

    fn engine(corp: &Arc<Vec<Vec<u8>>>) -> MutationEngine {
        let mut me = MutationEngine::new()
            .set_corpus(corp.clone())
            .set_generator(&Generators::Romuduojr)
            .set_generator_seed(0xdeadbeefcafebabe)
            .set_token_dict("dicts/test.dict");
        for _ in 0..128 {
            let tc_size = me.prng.rand_range(1, 4096);
            let tc = me.prng.rand_byte_vec(tc_size);
            me.add_to_corpus(&tc);
        }
        me
    }

    fn run<F>(fun: F, tcond: TestCondition)
    where
        F: Fn(&mut MutationEngine) -> Result<()>,
    {
        let corpus = corpus();
        let mut engine = engine(&corpus);

        for _ in 0..100_000 {
            engine = engine.set_random_test_case();
            let tc_orig = engine.test_case.data.clone();
            if fun(&mut engine).is_ok() {
                let tc = engine.test_case.data.clone();

                match tcond {
                    TestCondition::DataInequality => {
                        assert_ne!(tc_orig, tc);
                    }
                    TestCondition::SizeInequality => {
                        assert_ne!(tc_orig.len(), tc.len());
                    }
                    TestCondition::GeneralErrorChecker => {}
                };
            }
        }
    }

    #[test]
    fn test_shuffle_bytes() {
        // We check against `TestCondition::GeneralErrorChecker` as a low shuffle amount, with
        // a possible small slice to shuffle may yield the same result.
        // We don't care about the `sub_slice` being potentially the same.
        // We favor executions/second over ensuring the mutated version is != the original.
        run(
            MutationEngine::shuffle_bytes,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_erase_bytes() {
        run(MutationEngine::erase_bytes, TestCondition::SizeInequality);
    }

    #[test]
    fn test_insert_bytes() {
        run(MutationEngine::insert_bytes, TestCondition::SizeInequality);
    }

    #[test]
    fn test_swap_endianness() {
        // We check against `TestCondition::GeneralErrorChecker` as a low swap amount, with
        // a possible small slice to swap may yield the same result, for example in such a
        // scenario:
        //
        // `Swapping 2 bytes - [249, 249]`
        run(
            MutationEngine::swap_endianness,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_change_bit() {
        run(MutationEngine::change_bit, TestCondition::DataInequality);
    }

    #[test]
    fn test_change_byte() {
        run(MutationEngine::change_byte, TestCondition::DataInequality);
    }

    #[test]
    // Same argumentation as for `swap_endianness`.
    // The newly generated ASCII character can still be the same as before.
    fn test_change_ascii_integer() {
        run(
            MutationEngine::change_ascii_integer,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    // Same argumentation as for `swap_endianness`.
    // The done type casting can produce a value which the original data slice
    // at the rolled index already holds.
    fn test_change_binary_integer() {
        run(
            MutationEngine::change_binary_integer,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_negate_byte() {
        run(MutationEngine::negate_byte, TestCondition::DataInequality);
    }

    #[test]
    fn test_swap_neighbors() {
        // Same argumentation as for `swap_endianness`.
        run(
            MutationEngine::swap_neighbors,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_arithmetic() {
        // Same argumentation as for `swap_endianness`.
        run(
            MutationEngine::arithmetic_width,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_truncate() {
        run(MutationEngine::truncate, TestCondition::SizeInequality);
    }

    #[test]
    fn test_append() {
        run(MutationEngine::append, TestCondition::SizeInequality);
    }

    #[test]
    fn test_add_from_dict() {
        // Same argumentation as for `swap_endianness`.
        run(
            MutationEngine::add_word_from_dict,
            TestCondition::DataInequality,
        );
    }

    #[test]
    fn test_add_from_magic() {
        // Same argumentation as for `swap_endianness`.
        run(
            MutationEngine::add_from_magic,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_copy_part() {
        run(MutationEngine::copy_part, TestCondition::DataInequality);
    }

    #[test]
    fn test_cross_over() {
        // Seems to sometimes due to bad PRNG luck produce an identical test_case.
        // May need to revist later, for now speed > slowing down constraints.
        run(
            MutationEngine::cross_over,
            TestCondition::GeneralErrorChecker,
        );
    }

    #[test]
    fn test_splice() {
        // On bad rolls when two small test cases are selected, the splice may not
        // actually change the test case.
        run(MutationEngine::splice, TestCondition::GeneralErrorChecker);
    }

    #[test]
    fn test_ni() {
        let corpus: Arc<Vec<Vec<u8>>> = Arc::new(vec!["
<!DOCTYPE html>
  <html>
    <body>
      <h1>My 1337 Heading</h1>
      <p>My first paragraph.</p>
    </body>
  </html>"
            .as_bytes()
            .to_vec()]);

        let mut me = MutationEngine::new().set_corpus(corpus.clone());
        me = me.set_random_test_case();
        let _ = me.ni();
        assert_ne!(corpus[0], me.test_case.data);
    }

    #[test]
    #[ignore]
    fn test_torc() {}
}
