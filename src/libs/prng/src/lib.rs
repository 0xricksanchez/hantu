#![feature(test)]
extern crate test;
use clap::ValueEnum;
use core::ops::Deref;

pub mod lehmer;
pub mod romuduojr;
pub mod romutrio;
pub mod seed;
pub mod shishua;
pub mod splitmix;
pub mod wyhash;
pub mod xorshift;
pub mod xorshiro128ss;
pub mod xorshiro256ss;
use lehmer::Lehmer64;
use romuduojr::RomuDuoJr;
use romutrio::RomuTrio;
use shishua::ShiShua;
use splitmix::SplitMix64;
use wyhash::Wyhash64;
use xorshift::Xorshift64;
use xorshiro128ss::XorShiro128ss;
use xorshiro256ss::XorShiro256ss;

// Arbitrary value used for an initial entropy to seed our PRNG.
pub const ENTROPY: usize = 0x5fd8_9eda_3130_256d;
// A fixed list of special characters that we can use to generate random strings.
const SPECIAL_CHAR: [char; 30] = [
    '!', '*', '\'', '(', ')', ';', ':', '@', '&', '=', '+', '$', ',', '/', '?', '%', '#', '[', ']',
    '0', '1', '2', 'A', 'z', '-', '`', '~', '.', '\x7f', '\x00',
];

pub trait GeneratorTrait {
    fn rand(&mut self) -> usize;
    fn set_seed(&mut self, seed: usize);
}

#[derive(Clone, Debug)]
// Disable that clippy warning as we only ever have one Generator in memory at a time.
#[allow(clippy::large_enum_variant)]
pub enum Generator {
    Xorshift64(Xorshift64),
    RomuDuoJr(RomuDuoJr),
    RomuTrio(RomuTrio),
    SplitMix64(SplitMix64),
    XorShiro128ss(XorShiro128ss),
    XorShiro256ss(XorShiro256ss),
    Lehmer64(Lehmer64),
    Wyhash64(Wyhash64),
    ShiShua(ShiShua),
}

impl Default for Generator {
    fn default() -> Self {
        Self::RomuDuoJr(RomuDuoJr::new(0))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, ValueEnum)]
pub enum Generators {
    Xorshift64,
    Romuduojr,
    Romutrio,
    Splitmix64,
    Xorshiro128ss,
    Xorshiro256ss,
    Lehmer64,
    Wyhash64,
    Shishua,
}

impl Default for Generators {
    fn default() -> Self {
        Self::Romuduojr
    }
}

// If we ever add more methods / more generators and find this too boilerplatey to write,
// then we should try to look into the `enum_dispatch` crate.
impl GeneratorTrait for Generator {
    fn rand(&mut self) -> usize {
        match self {
            Self::Xorshift64(g) => g.rand(),
            Self::RomuDuoJr(g) => g.rand(),
            Self::RomuTrio(g) => g.rand(),
            Self::SplitMix64(g) => g.rand(),
            Self::XorShiro128ss(g) => g.rand(),
            Self::XorShiro256ss(g) => g.rand(),
            Self::Lehmer64(g) => g.rand(),
            Self::Wyhash64(g) => g.rand(),
            Self::ShiShua(g) => g.rand(),
        }
    }

    fn set_seed(&mut self, seed: usize) {
        match self {
            Self::Xorshift64(g) => g.set_seed(seed),
            Self::RomuDuoJr(g) => g.set_seed(seed),
            Self::RomuTrio(g) => g.set_seed(seed),
            Self::SplitMix64(g) => g.set_seed(seed),
            Self::XorShiro128ss(g) => g.set_seed(seed),
            Self::XorShiro256ss(g) => g.set_seed(seed),
            Self::Lehmer64(g) => g.set_seed(seed),
            Self::Wyhash64(g) => g.set_seed(seed),
            Self::ShiShua(g) => g.set_seed(seed),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rng<G> {
    pub exponential: bool,
    pub generator: G,
}

impl<G> Rng<G>
where
    G: GeneratorTrait,
{
    /// Creates a new `Rng` with the given generator `G`.
    pub fn new(generator: G) -> Self {
        Self {
            exponential: false,
            generator,
        }
    }

    /// Enables or disables the exponential distribution.
    /// Only used in `rand_range`.
    pub fn set_rand_exp(mut self, exp_enabled: bool) -> Self {
        self.exponential = exp_enabled;
        self
    }

    /// Sets the seed of the PRNG.
    pub fn set_seed(&mut self, seed: usize) {
        self.generator.set_seed(seed);
    }

    /// Sets the generator that will be used to generate random numbers.
    pub fn set_generator(mut self, generator: G) -> Self {
        self.generator = generator;
        self
    }

    /// Generates a random `usize` value using the generator's implementation of `GeneratorTrait`.
    ///
    /// # Returns
    ///
    /// A random `usize` value generated by the generator.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_value = prng.rand();
    /// // `random_value` is a random usize generated by the generator.
    /// ```
    #[inline]
    pub fn rand(&mut self) -> usize {
        self.generator.rand()
    }

    /// Generates a random number following a Gaussian distribution.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum possible value.
    /// * `max` - The maximum possible value.
    /// * `mean` - The mean (average) value of the distribution.
    /// * `stddev` - The standard deviation of the distribution. If not provided, the default value is `(max - min) / 2.0`.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    /// let gaussian_rand = prng.rand_gaussian(0.0, 100.0, 50.0, None);
    /// assert!(gaussian_rand >= 0.0 && gaussian_rand <= 100.0);
    /// ```
    ///
    /// This will generate a random number following a Gaussian distribution with mean `50.0`, minimum value `0.0`, and maximum value `100.0`.
    #[inline]
    pub fn rand_gaussian(&mut self, min: f64, max: f64, mean: f64, stddev: Option<f64>) -> f64 {
        assert!(max > min, "Failed bounds check in `rand_gaussian`");
        let stddev_ = stddev.map_or_else(|| (max - min) / 2.0, |x| x);
        let mut normal = (self.rand() as f64) / (core::usize::MAX as f64);
        normal = normal.mul_add(2.0_f64, 1.0_f64);
        normal *= stddev_;
        normal += mean;
        normal.clamp(min, max)
    }

    /// Generates a vector of `n` random numbers following a Gaussian distribution.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of random numbers to generate.
    /// * `min` - The minimum possible value.
    /// * `max` - The maximum possible value.
    /// * `mean` - The mean (average) value of the distribution.
    /// * `stddev` - The standard deviation of the distribution. If not provided, the default value is `(max - min) / 2.0`.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    /// let gaussian_vector = prng.rand_gaussian_n(10, 0.0, 100.0, 50.0, None);
    /// for gaussian in gaussian_vector {
    ///     assert!(gaussian >= 0.0 && gaussian <= 100.0);
    /// }
    /// ```
    ///
    /// This will generate a vector of `10` random numbers following a Gaussian distribution with mean `50.0`, minimum value `0.0`, and maximum value `100.0`.
    pub fn rand_gaussian_n(
        &mut self,
        n: usize,
        min: f64,
        max: f64,
        mean: f64,
        stddev: Option<f64>,
    ) -> Vec<f64> {
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(self.rand_gaussian(min, max, mean, stddev));
        }
        vec
    }

    /// Generates a random value of type `T` in the given range, possibly biased towards smaller values.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the output value, which must implement `Add`, `Sub`, `Rem`, `PartialOrd`,
    ///   `PartialEq`, `Display`, `Copy`, and `From<usize>`.
    ///
    /// # Arguments
    ///
    /// * `min`: The inclusive lower bound of the range.
    /// * `max`: The exclusive upper bound of the range.
    ///
    /// # Returns
    ///
    /// A random value of type `T` in the range `[min, max)`. If the `exponential` property of the
    /// generator is set, the distribution may be biased towards smaller values.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let min = 1;
    /// let max = 10;
    /// let random_value = prng.rand_exp(min, max);
    /// assert!(random_value >= min && random_value < max);
    /// ```
    #[inline]
    pub fn rand_exp<T>(&mut self, min: T, max: T) -> T
    where
        T: core::ops::Add<Output = T>
            + core::ops::Sub<Output = T>
            + core::ops::Rem<Output = T>
            + core::cmp::PartialOrd
            + core::cmp::PartialEq
            + core::fmt::Display
            + Copy
            + From<usize>,
    {
        if !self.exponential {
            return self.rand_range(min, max);
        }

        if self.bool() {
            self.rand_range(min, max)
        } else {
            let x = self.rand_range(min, max);
            self.rand_range(min, x)
        }
    }

    /// Generates 2 random `usize` in the range [0, max).
    ///
    /// # Arguments
    ///
    /// * `max` - The upper bound of the range (exclusive).
    ///
    /// # Returns
    ///
    /// Two random `usize`  in the specified range.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    /// let (a,b) = prng.rand_two(100);
    /// assert!(a < 100 && b < 100);
    #[inline]
    pub fn rand_two(&mut self, max: usize) -> (usize, usize) {
        if max <= 1 {
            return (0, 1);
        }
        let mut val_a = self.rand() % max;
        let mut val_b = self.rand() % max;

        loop {
            if val_a != val_b {
                break;
            }
            val_a = self.rand() % max;
            val_b = self.rand() % max;
        }

        if val_a > val_b {
            (val_b, val_a)
        } else {
            (val_a, val_b)
        }
    }

    /// Generate a random `T` in the range [low, high).
    ///
    /// # Arguments
    ///
    /// * `low` - The lower bound of the range (inclusive).
    /// * `high` - The upper bound of the range (exclusive).
    ///
    /// # Returns
    ///
    /// A random `T`  in the specified range.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    /// let num = prng.rand_range::<usize>(usize::MIN, usize::MAX);
    /// assert!(num < usize::MAX && num > usize::MIN);
    #[inline]
    pub fn rand_range<T>(&mut self, min: T, max: T) -> T
    where
        T: core::ops::Add<Output = T>
            + core::ops::Sub<Output = T>
            + core::ops::Rem<Output = T>
            + core::cmp::PartialOrd
            + core::cmp::PartialEq
            + core::fmt::Display
            + Copy
            + From<usize>,
    {
        assert!(
            max >= min,
            "Failed bounds check in `rand_range: max {max} < min {min}"
        );
        if min == max {
            return min;
        }
        min + T::from(self.rand()).rem(max - min)
    }

    /// Generate a random byte with the current generator.
    ///
    /// # Returns
    ///
    /// A random `u8`.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    /// let b = prng.rand_byte();
    /// assert!(b < 255 && b > 0);
    #[inline]
    pub fn rand_byte(&mut self) -> u8 {
        (self.rand() % 255) as u8
    }

    /// Picks a random item from a given iterable `entries` of `T` items
    /// and returns a copy of it.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the items in the iterable.
    /// * `I`: The type of the iterable, implementing `IntoIterator<Item = T> + Copy`.
    ///
    /// # Arguments
    ///
    /// * `entries`: The iterable containing the items to choose from.
    ///
    /// # Returns
    ///
    /// A randomly chosen copy of an item of type `T` from the provided iterable.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let choices = vec![1, 2, 3, 4, 5];
    /// let random_pick = prng.pick(&choices);
    /// assert!(choices.contains(&random_pick));
    /// ```
    #[inline]
    pub fn pick<T, I>(&mut self, entries: I) -> T
    where
        T: Clone,
        I: IntoIterator<Item = T>,
    {
        let mut entries_iter = entries.into_iter();
        let size_hint = entries_iter.size_hint();
        let idx = self.rand_range(0, size_hint.0);
        entries_iter.nth(idx).unwrap()
    }

    /// Picks a random item from a given iterable `entries` of `T` items
    /// and returns a reference of it.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the items in the iterable.
    ///
    /// # Arguments
    ///
    /// * `entries`: The iterable containing the items to choose from.
    ///
    /// # Returns
    ///
    /// A randomly chosen reference of an item of type `T` from the provided iterable.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let choices = vec![1, 2, 3, 4, 5];
    /// let random_pick = prng.pick(&choices);
    /// assert!(choices.contains(&random_pick));
    /// ```
    #[inline]
    pub fn pick_ref<'a, T>(&mut self, entries: &'a [T]) -> &'a T {
        let idx = self.rand_range(0, entries.len());
        &entries[idx]
    }

    /// Generates a random boolean value with equal probability of being `true` or `false`.
    ///
    /// # Returns
    ///
    /// A random boolean value.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_bool = prng.bool();
    /// ```
    #[inline]
    pub fn bool(&mut self) -> bool {
        0 == self.rand() % 2
    }

    /// Generates a random boolean value with a specified probability of being `true`.
    ///
    /// # Arguments
    ///
    /// * `prob`: The probability of the result being `true`, with `1` being equal to 100%.
    ///
    /// # Returns
    ///
    /// A random boolean value with the specified probability of being `true`.
    ///
    /// # Panics
    ///
    /// This function will panic if `prob` is `0`.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_bool = prng.bool_chance(4);
    /// ```
    #[inline]
    pub fn bool_chance(&mut self, prob: usize) -> bool {
        assert!(0 < prob, "Probability must be greater than 0");
        0 == self.rand_range(0, prob)
    }

    /// Generates a random byte vector of the specified size.
    ///
    /// # Arguments
    ///
    /// * `size`: The desired size of the output vector.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` of the specified size with randomly generated bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_vec = prng.rand_byte_vec(5);
    /// assert_eq!(random_vec.len(), 5);
    /// ```
    #[inline]
    pub fn rand_byte_vec(&mut self, size: usize) -> Vec<u8> {
        let mut v = vec![0_u8; size];
        v.fill_with(|| self.rand_byte());
        v
    }

    /// Generates a vector of unique random usize values in the specified range.
    ///
    /// # Arguments
    ///
    /// * `min`: The inclusive lower bound of the range.
    /// * `max`: The exclusive upper bound of the range.
    /// * `size`: The desired size of the output vector.
    ///
    /// # Returns
    ///
    /// A `Vec<usize>` of the specified size with randomly generated unique usize values within the given range.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_vec = prng.rand_range_vec(1, 6, 3);
    /// assert_eq!(random_vec.len(), 3);
    /// for value in &random_vec {
    ///     assert!(*value >= 1 && *value < 6);
    /// }
    /// ```
    #[inline]
    pub fn rand_range_vec(&mut self, min: usize, max: usize, size: usize) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::with_capacity(size);
        while v.len() != size {
            let b = self.rand_range(min, max);
            if !v.contains(&b) {
                v.push(b);
            }
        }
        v
    }

    /// Generate a random float value in the range `[0, 1]`.
    ///
    /// # Returns
    ///
    /// A random float value of type `f64` in the range `[0, 1]`.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_float = prng.rand_float::<f64>();
    /// assert!(random_float >= 0.0 && random_float <= 1.0);
    /// ```
    #[inline]
    pub fn rand_float<T: From<f64>>(&mut self) -> f64 {
        self.rand() as f64 / (usize::MAX as f64 + 1.0_f64)
    }

    /// Shuffle a vector of `T` entries.
    ///
    /// # Arguments
    ///
    /// * `entries`: A mutable reference to a slice of items to be shuffled.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let mut data = vec![1, 2, 3, 4, 5];
    /// let original_data = data.clone();
    /// prng.shuffle(&mut data);
    /// assert_ne!(data, original_data);
    /// // The data slice has been shuffled, so the order is different, but it still contains the same elements.
    /// ```
    #[inline]
    pub fn shuffle<T: Copy + core::fmt::Debug>(&mut self, entries: &mut [T]) {
        let len = entries.len();
        if len == 2 {
            entries.swap(0, 1);
            return;
        }
        for i in (1..len).rev() {
            let j = self.rand_range(0, i + 1);
            entries.swap(i, j);
        }
    }

    /// Return a random character from the set of alphanumeric characters and special characters, or
    /// a random byte with equal probability.
    ///
    /// # Returns
    ///
    /// A random `u8` value, which represents either a random character or a random byte.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let random_char = prng.rand_char();
    /// // `random_char` is either a random character or a random byte.
    /// ```
    #[inline]
    pub fn rand_char(&mut self) -> u8 {
        if self.bool() {
            return self.rand_byte();
        }
        SPECIAL_CHAR[self.rand_range(0, SPECIAL_CHAR.len())] as u8
    }

    /// Return `n` random indices from a vector of `T` entries.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type implementing `Deref<Target = [U]>`.
    /// * `U`: The type of the items in the vector.
    ///
    /// # Arguments
    ///
    /// * `entries`: A reference to the vector of items.
    /// * `n`: The number of unique random indices to be returned.
    ///
    /// # Returns
    ///
    /// A `Vec<usize>` containing `n` unique random indices from the given vector.
    ///
    /// # Example
    ///
    /// ```
    /// use prng::xorshift::Xorshift64;
    /// use prng::{Generator, Rng};
    /// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    /// let random_indices = prng.choose_multiple(&data, 3);
    /// assert_eq!(random_indices.len(), 3);
    /// for idx in random_indices {
    ///     assert!(idx < data.len());
    /// }
    /// ```
    #[inline]
    pub fn choose_multiple<T: Deref<Target = [U]>, U: core::marker::Sized>(
        &mut self,
        entries: &T,
        n: usize,
    ) -> Vec<usize> {
        let len = entries.len();
        let mut selected_indices = Vec::new();
        while selected_indices.len() < n {
            let idx = self.rand_range(0, len);
            if !selected_indices.contains(&idx) {
                selected_indices.push(idx);
            }
        }
        selected_indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::{black_box, Bencher};

    const SEED: usize = 0xb3959f04cb8af237;

    #[bench]
    pub fn xorshift64_bench(b: &mut Bencher) {
        let mut prng = Xorshift64::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn splitmix64_bench(b: &mut Bencher) {
        let mut prng = SplitMix64::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn romuduojr_bench(b: &mut Bencher) {
        let mut prng = RomuDuoJr::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn romutrio_bench(b: &mut Bencher) {
        let mut prng = RomuTrio::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn xorshiro256ss_bench(b: &mut Bencher) {
        let mut prng = XorShiro256ss::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn lehmer64_bench(b: &mut Bencher) {
        let mut prng = Lehmer64::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn wyhash64_bench(b: &mut Bencher) {
        let mut prng = Wyhash64::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }

    #[bench]
    pub fn shishua_bench(b: &mut Bencher) {
        let mut prng = ShiShua::new(SEED);
        b.iter(|| {
            for _ in 0..1_000_000 {
                black_box(prng.rand());
            }
        });
    }
}
