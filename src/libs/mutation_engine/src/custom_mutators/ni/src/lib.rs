use errors::Result;
use prng::{Generator, Rng};
use rayon::prelude::*;
use std::io::Write;
use std::sync::{Arc, Mutex};

// Implementation of <https://github.com/aoh/ni>

const AIMAX: usize = 512;
const AIMROUNDS: usize = 256;
const AIMLEN: usize = 1024;

/// Calculate the score of the difference between two byte slices `a` and `b`.
///
/// The score is calculated by iterating through the elements of the slices, comparing them, and
/// updating the score based on the differences found. The calculation stops when either the maximum
/// allowed score (`AIMAX`) is reached or when a matching element is found in both slices.
///
/// # Arguments
///
/// * `a`: A byte slice to compare.
/// * `b`: Another byte slice to compare.
///
/// # Returns
///
/// * The score of the difference between the two input slices.
fn sufscore(a: &[u8], b: &[u8]) -> usize {
    let mut n = 0;
    let mut last = u8::MAX;
    a.iter()
        .zip(b.iter())
        .take_while(|(a, b)| {
            if n >= AIMAX || *a == *b {
                false
            } else {
                if **a != last {
                    n += 32;
                }
                last = **a;
                true
            }
        })
        .count();
    n
}

/// Choose the best jump and land indices based on the input byte slices `from` and `to`.
///
/// The function tries to find the best pair of indices (jump, land) that optimizes the score
/// calculated by the `sufscore` function. The search is performed within the input byte slices
/// `from` and `to` and updates the mutable references to `jump` and `land`.
///
/// # Arguments
///
/// * `from`: A byte slice used as the source.
/// * `to`: A byte slice used as the target.
/// * `jump`: A mutable reference to a usize that will be set to the optimal jump index.
/// * `land`: A mutable reference to a usize that will be set to the optimal land index.
/// * `prng`: A mutable reference to a random number generator.
fn aim(from: &[u8], to: &[u8], jump: &mut usize, land: &mut usize, prng: &mut Rng<Generator>) {
    // Set jump and land to 0 if `from` is empty, otherwise set them to random values within the range of `from` length.
    let flen = from.len();
    let tlen = to.len();
    if flen == 0 {
        *jump = 0;
        *land = if tlen > 0 {
            prng.rand_range(0, tlen)
        } else {
            0
        };
        return;
    }

    if tlen == 0 {
        *land = 0;
        return;
    }

    *jump = prng.rand_range(0, flen);
    *land = prng.rand_range(0, tlen);

    // Store the best score and update jump and land with better scores found
    let mut best_score = 0;
    let rounds = prng.rand_range(0, AIMROUNDS);
    for _ in 0..rounds {
        let mut maxs = AIMLEN;
        let j = prng.rand_range(0, flen);
        let mut l = prng.rand_range(0, tlen);
        while maxs > 0 && l < tlen && from[j] != to[l] {
            l += 1;
            maxs -= 1;
        }
        let score = sufscore(&from[j..], &to[l..]);
        if score > best_score {
            best_score = score;
            *jump = j;
            *land = l;
        }
    }
}

/// Generate a random block of bytes based on the input data and corpus.
///
/// The function selects a random sample from the corpus, and then chooses a random starting
/// position within the sample. It copies the bytes from the starting position to the end of the
/// sample, with the maximum length of the copied data being 4 times the length of the input data.
///
/// # Arguments
///
/// * `data`: A byte slice used as the input data.
/// * `prng`: A mutable reference to a random number generator.
/// * `corpus`: A reference-counted wrapper around a `Vec` containing a corpus of byte slices.
///
/// # Returns
///
/// * A `Vec<u8>` containing the randomly generated block of bytes.
fn random_block(data: &[u8], prng: &mut Rng<Generator>, corpus: &Arc<Vec<Vec<u8>>>) -> Vec<u8> {
    let other = corpus
        .get(prng.rand_range(0, corpus.len()))
        .map_or_else(|| prng.rand_byte_vec(4096), std::clone::Clone::clone);
    let olen = other.len();
    if olen < 3 {
        return data.to_vec();
    }
    let start = prng.rand_range(0, olen - 2);

    let mut len = olen - start;
    let dlen = data.len();
    if len > 4 * dlen {
        len = 4 * dlen;
    }
    len = prng.rand_range(0, len);
    other[len..].to_vec()
}

/// Search for a number in the input data and return the start and end indices of the number.
///
/// The function searches for a number starting from a random position within the input data.
/// If a number is found, the function returns a tuple with the start and end indices of the number.
///
/// # Arguments
///
/// * `data`: A byte slice used as the input data.
/// * `prng`: A mutable reference to a random number generator.
///
/// # Returns
///
/// * An `Option<(usize, usize)>` containing the start and end indices of the number if found, otherwise `None`.
fn seek_num(data: &[u8], prng: &mut Rng<Generator>) -> Option<(usize, usize)> {
    let end = data.len();
    if end == 0 {
        return None;
    }
    let mut o = prng.rand_range(0, end);
    while o < end && !data[o].is_ascii_digit() {
        if data[o] & 128 != 0 {
            return None;
        }
        o += 1;
    }
    if o == end {
        return None;
    }
    let ns = o;
    o += 1;
    while o < end && data[o].is_ascii_digit() {
        o += 1;
    }
    let ne = o;
    Some((ns, ne))
}

/// Twiddle the input value using random operations.
///
/// The function applies one of the following operations to the input value:
/// 1. Replace it with a new random i64 number.
/// 2. Flip one of its bits.
/// 3. Add a number relatively close to 0.
///
/// The function continues to apply random operations 50% of the time.
///
/// # Arguments
///
/// * `val`: The i64 value to be twiddled.
/// * `prng`: A mutable reference to a random number generator.
///
/// # Returns
///
/// * An `i64` representing the twiddled value.
fn twiddle(mut val: i64, prng: &mut Rng<Generator>) -> i64 {
    loop {
        match prng.rand_range(0, 3) {
            0 => {
                val = prng.rand() as i64;
            }
            1 => {
                val ^= 1 << prng.rand_range(0, (std::mem::size_of::<i64>() * 8) - 1) as i64;
            }
            2 => {
                val += prng.rand_range(0, 5) as i64 - 2;
            }
            _ => continue,
        }
        if prng.bool() {
            break;
        }
    }
    val
}

/// Returns the opposite delimiter for a given delimiter.
///
/// # Arguments
///
/// * `delim`: A `u8` representing the input delimiter.
///
/// # Returns
///
/// An `Option<u8>` containing the opposite delimiter as a `u8`. Returns `None` if the input delimiter is not one of the specified delimiters.
const fn delim_of(delim: u8) -> Option<u8> {
    match delim {
        b'<' => Some(b'>'),
        b'(' => Some(b')'),
        b'{' => Some(b'}'),
        b'[' => Some(b']'),
        b'>' => Some(b'<'),
        b')' => Some(b'('),
        b'}' => Some(b'{'),
        b']' => Some(b'['),
        b'\n' => Some(b'\n'),
        //b'"' => Some(b'"'),
        //b' ' => Some(b' '),
        //b',' => Some(b','),
        _ => None,
    }
}

/// Searches for the first delimiter in a given data slice and returns its index and a reference to the delimiter character.
///
/// Delimiters are considered to be one of the following characters: `[`, `<`, `(`, or `\n`.
///
/// # Arguments
///
/// * `data`: A reference to a slice of bytes to search for a delimiter.
///
/// # Returns
///
/// An `Option<(usize, &u8)>`, where the first element is the index of the delimiter and the second element is a reference to the delimiter character.
/// Returns `None` if no delimiter is found in the data slice.
fn drange_start(data: &[u8]) -> Option<(usize, &u8)> {
    data.iter()
        .enumerate()
        .find(|&(_, c)| matches!(*c as char, '[' | '<' | '(' | '\n'))
        .map(|(i, c)| (i, c))
}

/// Finds the position of the closing delimiter in a slice of data, considering nested delimiters.
///
/// # Arguments
///
/// * `data`: A slice of `u8` data to search for the closing delimiter.
/// * `delim_open`: The opening delimiter as a `u8`.
/// * `delim_close`: The closing delimiter as a `u8`.
/// * `stop_chance`: The chance to stop searching for the next closing delimiter, as a fraction of 1.
///
/// # Returns
///
/// An `Option<usize>` containing the position of the closing delimiter if found, otherwise `None`.
fn drange_end(
    data: &[u8],
    delim_open: u8,
    delim_close: u8,
    prng: &mut Rng<Generator>,
) -> Option<usize> {
    let mut depth = 0;

    for (i, c) in data.iter().enumerate() {
        if *c == delim_close {
            depth -= 1;
            if depth == 0 {
                if prng.bool_chance(3) {
                    return Some(i + 1);
                }

                let next = drange_end(&data[i + 1..], delim_open, delim_close, prng);
                match next {
                    Some(x) => return Some(i + 1 + x),
                    None => return Some(i + 1),
                }
            }
        } else if *c == delim_open {
            depth += 1;
        } else if c & 128 > 0 {
            return None;
        }
    }
    None
}

/// Finds the range of data enclosed by a matching pair of delimiters, considering nested delimiters.
///
/// This function takes a mutable reference to a custom random number generator (`Rng<Generator>`).
///
/// # Arguments
///
/// * `data`: A slice of `u8` data to search for the delimiters.
/// * `prng`: A mutable reference to a custom random number generator.
///
/// # Returns
///
/// An `Option<(usize, usize)>` containing the start and end positions of the range if found, otherwise `None`.
fn drange(data: &[u8], prng: &mut Rng<Generator>) -> Option<(usize, usize)> {
    let (delim_start, delim) = drange_start(data)?;
    let delim_close = delim_of(*delim)?;
    let delim_end = drange_end(&data[delim_start..], *delim, delim_close, prng)?;
    Some((delim_start, delim_start + delim_end))
}

/// Finds the range of data enclosed by a specified opening delimiter and its corresponding closing delimiter.
///
/// This function takes a mutable reference to a custom random number generator (`Rng<Generator>`).
///
/// # Arguments
///
/// * `data`: A slice of `u8` data to search for the specified opening delimiter.
/// * `delim_start`: The opening delimiter to search for in the data.
/// * `prng`: A mutable reference to a custom random number generator.
///
/// # Returns
///
/// An `Option<(usize, usize)>` containing the start and end positions of the range if found, otherwise `None`.
fn other_drange(data: &[u8], delim_start: u8, prng: &mut Rng<Generator>) -> Option<(usize, usize)> {
    let delim_close = delim_of(delim_start)?;
    for _ in 0..10 {
        let start = prng.rand_range(0, data.len());
        let temp_data = &data[start..];
        for (i, c) in temp_data.iter().enumerate() {
            if *c == delim_start {
                let delim_end = drange_end(&temp_data[i..], delim_start, delim_close, prng);
                match delim_end {
                    None => continue,
                    Some(end) => {
                        return Some((start + i, start + i + end));
                    }
                }
            }
        }
    }
    None
}

/// Mutates a given data slice according to various mutation strategies and writes the result to the output.
///
/// This function accepts a reference to a data slice, a mutable reference to an output writer implementing the Write trait,
/// a mutable reference to a Rng instance with a Generator, and a shared reference to a corpus.
///
/// The function applies various mutation strategies to the input data, including but not limited to:
/// - Inserting or deleting random bytes
/// - Jumping or overlapping sequences
/// - Repeating characters
/// - Inserting random data
/// - Aimed jump to self
/// - Aimed random block fusion
/// - Inserting or overwriting semi-random bytes
/// - Textual number mutation
/// - Delimiter swapping
///
/// # Arguments
///
/// * data - A slice of input data to be mutated
/// * out - A mutable reference to the output writer implementing the Write trait, where the mutated result will be written
/// * prng - A mutable reference to a Rng instance with a Generator, used to generate random values for the mutation strategies
/// * corpus - A shared reference to a corpus, used in some mutation strategies for reference data
fn mutate_area<W: Write>(
    data: &[u8],
    out: &mut W,
    prng: &mut Rng<Generator>,
    corpus: &Arc<Vec<Vec<u8>>>,
) {
    let end = data.len();
    loop {
        let r = prng.rand_range(0, 35);
        match r {
            0 => {
                // Insert random byte
                let pos = prng.rand_range(0, end);
                let _ = out.write(&data[..pos]);
                let _ = out.write(&prng.rand_byte_vec(1));
                let _ = out.write(&data[pos..]);
                return;
            }
            1 => {
                // Delete a random byte
                let pos = prng.rand_range(0, end);
                if pos + 1 >= end {
                    continue;
                }
                let _ = out.write(&data[..pos]);
                let _ = out.write(&data[pos + 1..]);
                return;
            }
            2..=3 => {
                // Jump / Overlapping sequences
                if end <= 1 {
                    continue;
                }
                // Generate two random numbers a,b, with a<b
                let (a, b) = prng.rand_two(end);
                let _ = out.write(&data[..a]);
                let _ = out.write(&data[b..]);
                return;
            }
            4..=5 => {
                // Repeat characters
                if end < 2 {
                    continue;
                }
                let mut n = 8;
                while prng.bool() && n < 20000 {
                    n <<= 1;
                }
                n = prng.rand_range(1, n + 3);
                let (a, b) = prng.rand_two(end);
                let mut len = b - a;

                let _ = out.write(&data[..a]);

                if len * n > 0x800_0000 {
                    len = prng.rand_range(0, 1026);
                }

                // Insert some substring `n` times
                for _ in 0..n {
                    let _ = out.write(&data[a..a + len]);
                }

                let _ = out.write(&data[a..]);
                return;
            }
            6 => {
                // Insert random data
                let pos = prng.rand_range(0, end);
                let n = prng.rand_range(0, 1024);
                let random_data = prng.rand_byte_vec(n);
                let _ = out.write(&data[..pos]);
                let _ = out.write(&random_data);
                let _ = out.write(&data[pos..]);
                return;
            }
            7..=12 => {
                // Aimed jump to self
                if end < 5 {
                    continue;
                }

                let mut j = 0;
                let mut l = 0;
                aim(data, data, &mut j, &mut l, prng);

                let _ = out.write(&data[..j]);
                let _ = out.write(&data[l..]);
                return;
            }
            13..=21 => {
                // Aimed random block fusion
                if end < 8 {
                    continue;
                }

                let rchk = random_block(data, prng, corpus);
                let mut j = 0;
                let mut l = 1;
                aim(
                    &data[..end >> 1],
                    &rchk[..rchk.len() >> 1],
                    &mut j,
                    &mut l,
                    prng,
                );
                let _ = out.write(&data[..j]);

                let buff = &rchk[rchk.len() >> 1..];
                aim(buff, &data[j..], &mut j, &mut l, prng);
                let _ = out.write(&buff[..j]);
                let _ = out.write(&data[l..]);
                return;
            }
            22..=23 => {
                // Insert semirandom bytes
                if end < 2 {
                    continue;
                }

                let n = prng.rand_range(2, 4096) % (4096 / 5);
                let pos = prng.rand_range(0, end);
                let mut r = prng.rand_range(2, data.len());
                let _ = out.write(&data[..pos]);
                for _ in 0..n {
                    let _ = out.write(&data[r - 1..r]);
                    r = prng.rand_range(2, data.len());
                }
                let _ = out.write(&data[pos..]);
                return;
            }
            24 => {
                // Overwrite semirandom bytes
                if end < 2 {
                    continue;
                }

                let a = prng.rand_range(0, end - 2);
                let mut b = a + 2;
                if prng.bool() {
                    b += prng.rand_range(0, 32);
                } else {
                    b += prng.rand_range(0, std::cmp::min(4096 - 2, end - a - 2));
                }
                b = std::cmp::min(b, end);

                let _ = out.write(&data[..a]);
                for _ in a..b {
                    let r = prng.rand_range(0, end);
                    let _ = out.write(&data[r..=r]);
                }

                if end > b {
                    let _ = out.write(&data[b..]);
                }
                return;
            }
            25..=28 => {
                // Textual number mutation
                if end < 2 {
                    continue;
                }
                // Attempt to find a number at a random location in the data buffer
                for _ in 0..prng.rand_range(0, AIMROUNDS) {
                    if let Some((ns, ne)) = seek_num(data, prng) {
                        let _ = out.write(&data[..ns]);
                        let num = std::str::from_utf8(&data[ns..ne])
                            .unwrap()
                            .parse::<usize>()
                            .unwrap() as i64;
                        let twid = twiddle(num, prng);
                        let raw_bytes: [u8; 8] = twid.to_ne_bytes();
                        let _ = out.write(&raw_bytes);
                        let _ = out.write(&data[ne..]);
                        break;
                    }
                }
                return;
            }
            29..=34 => {
                // delim swap
                match drange(data, prng) {
                    None => continue,
                    Some((delim1_s, delim1_e)) => match other_drange(data, data[delim1_s], prng) {
                        None => continue,
                        Some((delim2_s, delim2_e)) => {
                            let _ = out.write(&data[..delim1_s]);
                            let _ = out.write(&data[delim2_s..delim2_e]);
                            if delim2_s > delim1_e {
                                let _ = out.write(&data[delim1_e..delim2_s]);
                            }
                            let _ = out.write(&data[delim1_s..delim1_e]);
                            let _ = out.write(&data[delim2_e..]);
                        }
                    },
                }

                return;
            }
            _ => unimplemented!(),
        }
    }
}

/// Performs a mutation process on a given data slice.
/// It's using recursion to perform the mutation process and each
/// recursion step is performed in a separate thread.
///
/// This function takes a mutable reference to a custom random number generator (`Rng<Generator>`).
///
/// # Arguments
///
/// * `data`: A slice of `u8` data to process.
/// * `n`: The number of iterations of the mutation process.
/// * `out`: A mutable reference to a `Vec<u8>` to write the output to.
/// * `prng`: A mutable reference to a custom random number generator.
/// * `corpus`: A shared reference to an `Arc<Vec<Vec<u8>>>` containing the corpus data.
///
/// # Returns
///
/// A `Vec<u8>` containing the mutated data.
pub fn ni_area_parallel<W: Write + Send + Sync>(
    data: &[u8],
    n: usize,
    out: &mut W,
    prng: &mut Rng<Generator>,
    corpus: &Arc<Vec<Vec<u8>>>,
) {
    let len = data.len();

    if n == 1 || len < 256 {
        mutate_area(data, out, prng, corpus);
    } else {
        // Determine the number of threads based on the available hardware
        let num_threads = rayon::current_num_threads();
        let chunk_size = len / num_threads;

        // Create a shared Mutex for the output writer
        let out_mutex = Arc::new(Mutex::new(out));

        // Divide the data into equal-sized chunks and process them in parallel
        data.par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_prng = prng.clone();
                let mut local_out = vec![];
                ni_area_parallel(
                    chunk,
                    n / num_threads,
                    &mut local_out,
                    &mut local_prng,
                    corpus,
                );
                local_out
            })
            .for_each_with(out_mutex, |out_mutex, local_out| {
                let mut out = out_mutex.lock().unwrap();
                out.write_all(&local_out).unwrap();
            });
    }
}

/// Performs a mutation process on a given data slice.
/// It's using threaded recursion to perform the first stage of the mutation and all further
/// stagesi are done using a stack.
///
/// This function takes a mutable reference to a custom random number generator (`Rng<Generator>`).
///
/// # Arguments
///
/// * `data`: A slice of `u8` data to process.
/// * `n`: The number of iterations of the mutation process.
/// * `out`: A mutable reference to a `Vec<u8>` to write the output to.
/// * `prng`: A mutable reference to a custom random number generator.
/// * `corpus`: A shared reference to an `Arc<Vec<Vec<u8>>>` containing the corpus data.
///
/// # Returns
///
/// A `Vec<u8>` containing the mutated data.
pub fn ni_area_parallel_hybrid<W: Write + Send + Sync>(
    data: &[u8],
    n: usize,
    out: &mut W,
    prng: &mut Rng<Generator>,
    corpus: &Arc<Vec<Vec<u8>>>,
) {
    let len = data.len();

    if n == 1 || len < 256 {
        mutate_area(data, out, prng, corpus);
    } else {
        // Determine the number of threads based on the available hardware
        let num_threads = rayon::current_num_threads();
        let chunk_size = len / num_threads;

        // Create a shared Mutex for the output writer
        let out_mutex = Arc::new(Mutex::new(out));

        // Divide the data into equal-sized chunks and process them in parallel
        data.par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_prng = prng.clone();
                let mut local_out = vec![];
                ni_area(
                    chunk,
                    n / num_threads,
                    &mut local_out,
                    &mut local_prng,
                    corpus,
                );
                local_out
            })
            .for_each_with(out_mutex, |out_mutex, local_out| {
                let mut out = out_mutex.lock().unwrap();
                out.write_all(&local_out).unwrap();
            });
    }
}

/// This is the equivalent of `ni_area_parallel` but it uses a stack instead of recursion and no parallelism.
/// It solely exists for benchmarking purposes as it turned out that the parallel version is faster
/// across all tested input sizes between 1 and 1000000 bytes.
pub fn ni_area<W: Write>(
    data: &[u8],
    n: usize,
    out: &mut W,
    prng: &mut Rng<Generator>,
    corpus: &Arc<Vec<Vec<u8>>>,
) {
    let mut stack = vec![(data, n)];
    while let Some((data, n)) = stack.pop() {
        let len = data.len();
        if n == 1 || len < 256 {
            mutate_area(data, out, prng, corpus);
        } else {
            let mut split = prng.rand_range(0, len);
            while split == 1 {
                split = prng.rand_range(0, len);
            }
            let rng_max = prng.rand_range(0, n);
            let new_n = prng.rand_range(0, n - rng_max);
            stack.push((&data[split..], new_n));
            stack.push((&data[..split], n - new_n));
        }
    }
}

/// Mutate a randomly selected sample from a vector of samples using a custom random number generator.
///
/// # Arguments
///
/// * `samples`: A reference to a `Vec<Vec<u8>>` containing the samples.
/// * `data_sz`: The size of the data.
/// * `prng`: A mutable reference to a custom random number generator.
/// * `corpus`: A shared reference to an `Arc<Vec<Vec<u8>>>` containing the corpus data.
///
/// # Returns
///
/// A `Result<Vec<u8>>` containing the mutated data.
///
/// # Example
///
/// ```
/// use prng::xorshift::Xorshift64;
/// use prng::{Generator, Rng};
/// use std::sync::Arc;
/// use ni::ni_mutate;
/// let corpus: Arc<Vec<Vec<u8>>> = Arc::new(vec!["<!DOCTYPE html>
/// <html>
///   <body><h1>My 1337 Heading</h1>
///     <p>My first paragraph.</p>
///   </body>
/// </html>".as_bytes().to_vec(),
/// ]);
/// let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
/// let mut data = corpus[0].clone();
/// let data_sz = data.len();
/// let res = ni_mutate(&mut data, data_sz, &mut prng, &corpus).unwrap();
/// assert!(res.len() > 0);
/// assert_ne!(res, corpus[0]);
/// ```
pub fn ni_mutate(
    data: &[u8],
    data_sz: usize,
    prng: &mut Rng<Generator>,
    corpus: &Arc<Vec<Vec<u8>>>,
) -> Result<Vec<u8>> {
    let mut res = Vec::new();
    let n = if prng.rand() & 3 == 1 {
        1
    } else {
        2 + prng.rand_range(0, data_sz >> (12 + 8))
    };
    if data_sz < 4096 {
        ni_area(data, n, &mut res, prng, corpus);
    } else {
        ni_area_parallel_hybrid(data, n, &mut res, prng, corpus);
    }
    Ok(res)
}
