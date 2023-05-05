use clap::{builder::PossibleValuesParser, Parser};
use errors::Result;
use executor::{spawn_workers, FuzzerConfig, FuzzerStats};
use grammar_mutator::GrammarTemplate;
use prng::Generators;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Clargs {
    #[clap(
        last(true),
        required(true),
        help = "Target binary to fuzz including args. e.g. ./target -a -b -c. Append @@ to fuzz from file. e.g. ./target -a -b -c @@."
    )]
    target: Vec<String>,
    #[clap(
        short,
        long,
        default_value = "./.corpus",
        help = "A directory containing a seed corpus."
    )]
    #[arg(short = 'i')]
    corpus_dir: String,
    #[clap(
        short,
        long,
        default_value = "./.crashes",
        help = "A directory to store results, such as reproducible crashes."
    )]
    #[arg(short = 'o')]
    crash_dir: String,
    #[clap(short, long, default_value = None, help = "An optional dictionary file of newline separated entries that are used in the mutator")]
    user_dict: Option<String>,
    #[clap(short, long, default_value = None, help = "Maximum number of iterations to run for")]
    max_iter: Option<usize>,
    #[clap(short, long, default_value = "1", help = "Number of threads to use")]
    #[arg(short = 'n')]
    threads: usize,
    #[clap(short, long, default_value = "romuduojr", help = "PRNG to use")]
    #[arg(value_enum)]
    prng: Generators,
    #[clap(short, long, default_value = "0", help = "Seed for PRNG")]
    seed: usize,
    #[clap(long, default_value = None, help = "Enable an optional grammar generator for the mutator to create (semi)-valid inputs")]
    #[arg(value_name = "grammar", value_parser = PossibleValuesParser::new(&GrammarTemplate::NAMES))]
    grammar_mutator: Option<String>,
    #[clap(long, help = "Enable the optional ni mutator")]
    ni_mutator: bool,
    #[clap(
        long,
        help = "Enforce the generated test cases to only contain printable characters"
    )]
    printable: bool,
    #[clap(
        long,
        default_value = "1",
        help = "Number of mutations to apply to each test case"
    )]
    mutation_passes: usize,
    #[clap(
        short,
        long,
        default_value = "1000",
        help = "Iterations before updating stats"
    )]
    batch_sz: usize,
}

impl From<Clargs> for FuzzerConfig {
    fn from(args: Clargs) -> Self {
        Self::default()
            .set_target(args.target)
            .set_corpus_dir(&args.corpus_dir)
            .set_crash_dir(&args.crash_dir)
            .set_threads(args.threads)
            .set_batch_sz(args.batch_sz)
            .set_seed(args.seed)
            .set_generator(args.prng)
            .set_ni_mutator(args.ni_mutator)
            .set_dict(args.user_dict)
            .set_max_iter(args.max_iter)
            .set_grammar(args.grammar_mutator)
            .set_printable(args.printable)
            .set_mutation_passes(args.mutation_passes)
    }
}

fn main() -> Result<()> {
    let fuzzer_config: FuzzerConfig = Clargs::parse().into();
    let fuzzer_stats = FuzzerStats::new().to_arc();
    println!("[HANTU] Using fuzing config: {fuzzer_config:#?}");

    spawn_workers(&fuzzer_config, &fuzzer_stats).unwrap_or_else(|e| {
        panic!("Error spawning workers: {e}");
    });
    let start_time = Instant::now();

    std::thread::sleep(std::time::Duration::from_secs(1));
    loop {
        let elapsed = start_time.elapsed().as_secs_f64();
        let iterations = fuzzer_stats.get_iterations();
        let crashes = fuzzer_stats.get_crashes();
        let execs_per_sec = iterations as f64 / elapsed;
        println!(
            "[{:10.6}] Iterations: {:10} - exec/sec: {:8.1} - crashes: {:5}",
            elapsed, iterations, execs_per_sec, crashes
        );
        if let Some(max_iter) = fuzzer_config.max_iter {
            if iterations >= max_iter {
                break Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
