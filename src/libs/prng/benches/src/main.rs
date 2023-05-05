use image::{ImageBuffer, Rgb};
use prng::lehmer::Lehmer64;
use prng::romuduojr::RomuDuoJr;
use prng::romutrio::RomuTrio;
use prng::shishua::ShiShua;
use prng::splitmix::SplitMix64;
use prng::wyhash::Wyhash64;
use prng::xorshift::Xorshift64;
use prng::xorshiro128ss::XorShiro128ss;
use prng::xorshiro256ss::XorShiro256ss;
use prng::{Generator, Rng};
use statrs::distribution::ChiSquared;
use statrs::distribution::ContinuousCDF;

use plotters::prelude::*;
use statrs::statistics::Statistics;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};

const NUM_SAMPLES: usize = 10_000_000;
const NUM_BINS: usize = 100;
const BIN_SZ: usize = std::usize::MAX / NUM_BINS;

const SEED: usize = 0x1b31_38ac_0b0f_bab1;

fn shannon_entropy(data: &[u8]) -> f64 {
    let mut frequency_map = HashMap::new();
    for byte in data {
        *frequency_map.entry(byte).or_insert(0) += 1;
    }

    let len = data.len() as f64;
    frequency_map
        .values()
        .map(|count| {
            let p = f64::from(*count) / len;
            -p * p.log2()
        })
        .sum()
}

// Define a struct to hold information about each PRNG
struct PRNGInfo {
    name: &'static str,
    rng: Rng<Generator>,
    total_duration: Duration,
    obsv_freqs: Vec<usize>,
}

fn visualize_prng(prng_info: &mut PRNGInfo) {
    let img_sz = 1024 * 1024 * 3;
    let bv = prng_info.rng.rand_byte_vec(img_sz);

    let mut img = ImageBuffer::new(1024, 1024);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let offset: usize = (y as usize * 1024 + x as usize) * 3;
        let r = bv[offset];
        let g = bv[offset + 1];
        let b = bv[offset + 2];

        *pixel = Rgb([r, g, b]);
    }

    img.save(format!("{}_viz.png", prng_info.name)).unwrap();

    println!("=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-==-=-=-=-=-=-=");
    println!(
        "Entropy: {} with seed {} -> {:.2} bits per byte",
        prng_info.name,
        SEED,
        shannon_entropy(bv.as_slice())
    );
}

// Define a function to run a statistical test on a given PRNG and print the results
fn test_prng(prng_info: &mut PRNGInfo) {
    // Generate a large number of random numbers
    let start_time = Instant::now();
    let mut samples = Vec::with_capacity(NUM_SAMPLES);
    for _ in 0..NUM_SAMPLES {
        samples.push(prng_info.rng.rand_range(0, std::usize::MAX));
    }
    let duration = start_time.elapsed();

    // Compute the observed and expected frequencies of each value
    for sample in &samples {
        let bin = sample / BIN_SZ;
        prng_info.obsv_freqs[bin] += 1;
    }
    let expected_frequency = samples.len() / NUM_BINS;

    // Compute the chi-squared statistic and p-value
    let mut chi_squared = 0.0;
    for observed_frequency in &prng_info.obsv_freqs {
        chi_squared += (*observed_frequency as f64 - expected_frequency as f64).powi(2)
            / expected_frequency as f64;
    }
    let p_value = 1.0
        - ChiSquared::new(NUM_BINS as f64 - 1.0)
            .unwrap()
            .cdf(chi_squared);

    // Print the results
    println!("PRNG: {}", prng_info.name);
    println!("Chi-squared: {chi_squared}");
    println!("P-value: {p_value}");
    println!("Total duration: {duration:?}");
    println!(
        "Average runtime per generated random number: {:?}",
        duration / NUM_SAMPLES as u32
    );
    println!();

    // Update PRNGInfo struct with statistics
    prng_info.total_duration = duration;
}

fn main() {
    let args = env::args().count();
    // Define an array of PRNGs to test
    let mut prngs: [PRNGInfo; 9] = [
        PRNGInfo {
            name: "Xorshift64",
            rng: Rng::new(Generator::Xorshift64(Xorshift64::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "XorShiro128ss",
            rng: Rng::new(Generator::XorShiro128ss(XorShiro128ss::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "XorShiro256ss",
            rng: Rng::new(Generator::XorShiro256ss(XorShiro256ss::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "Lehmer64",
            rng: Rng::new(Generator::Lehmer64(Lehmer64::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "SplitMix64",
            rng: Rng::new(Generator::SplitMix64(SplitMix64::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "ShiShua",
            rng: Rng::new(Generator::ShiShua(ShiShua::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "Wyhash64",
            rng: Rng::new(Generator::Wyhash64(Wyhash64::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "RomuDuoJr",
            rng: Rng::new(Generator::RomuDuoJr(RomuDuoJr::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
        PRNGInfo {
            name: "RomuTrio",
            rng: Rng::new(Generator::RomuTrio(RomuTrio::new(SEED))),
            total_duration: Duration::default(),
            obsv_freqs: vec![0; NUM_BINS],
        },
    ];

    if args > 1 {
        for prng_info in &mut prngs {
            visualize_prng(prng_info);
        }
    } else {
        // Test each PRNG and print the results
        for prng_info in &mut prngs {
            test_prng(prng_info);
        }
        let min_freq = prngs
            .iter()
            .flat_map(|prng| prng.obsv_freqs.iter())
            .min()
            .unwrap();
        let max_freq = prngs
            .iter()
            .flat_map(|prng| prng.obsv_freqs.iter())
            .max()
            .unwrap();

        let bin_centers = (0..NUM_BINS)
            .map(|i| (i as f64 + 0.5) * BIN_SZ as f64)
            .collect::<Vec<_>>();

        let root = BitMapBackend::new("benched.png", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("PRNG benchmark (SEED: {SEED:#016x})"),
                ("sans-serif", 50).into_font(),
            )
            .margin(50)
            .x_label_area_size(70)
            .y_label_area_size(120)
            .build_cartesian_2d(
                bin_centers.clone().min()..bin_centers.clone().max(),
                *min_freq as f64..*max_freq as f64,
            )
            .unwrap();

        chart
            .configure_mesh()
            .x_desc(format!(
                "Observed 64-bit values over {NUM_SAMPLES} random samples (max of usize::MAX)"
            ))
            .y_desc("Value frequency")
            .x_label_formatter(&|v| format!("{:.1e}", v))
            .axis_desc_style(("sans-serif", 30))
            .label_style(("sans-serif", 20))
            .draw()
            .unwrap();

        // Plot all the things
        (0..prngs.len()).for_each(|i| {
            // Combine the x and y data points into a vector of (x, y) tuples
            let data = bin_centers
                .iter()
                .zip(prngs[i].obsv_freqs.iter())
                .map(|(&x, &y)| (x, y as f64))
                .collect::<Vec<_>>();
            let color = Palette99::pick(i).mix(0.9);
            chart
                .draw_series(LineSeries::new(data, color))
                .unwrap()
                .label(prngs[i].name)
                .legend(move |(x, y)| {
                    Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                });
        });
        chart
            .configure_series_labels()
            .margin(15)
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .label_font(("sans-serif", 16))
            .draw()
            .unwrap();

        root.present().unwrap();
    }
}
