# hantu

This is a basic single threaded fuzzing mutator that either generates random input from scratch or mutates files from a corpus.
Up to 10% (arbitrarily chosen) are mutated.
This in turn means, the larger the files in the input corpus the slower the mutation.
It probably would be nice to make the mutation maximum a tweakable while also adding a `corpus.trim()` option that removes dead weight.
For example such that do not produce more code coverage or improve any other fuzzing metric.
However, as this is up to the fuzzers core logic and not work of the mutator it's intentionally being left out here.

```
cd hantu_generator_test
cargo build --release
./target/release/hantu_generator_test
Hello, world!
Average test case size: 55782 bytes
Execs      10000 -    13892.4/s
Execs      20000 -    13604.8/s
Execs      30000 -    13576.9/s
Execs      40000 -    13824.9/s
Execs      50000 -    14108.3/s
Execs      60000 -    14170.7/s
Execs      70000 -    14085.7/s
Execs      80000 -    13976.9/s
Execs      90000 -    13931.6/s
Execs     100000 -    13859.9/s
Execs     110000 -    13869.8/s
Execs     120000 -    13886.5/s
Execs     130000 -    13935.4/s
Execs     140000 -    13997.7/s
Execs     150000 -    13998.0/s
Execs     160000 -    14080.1/s
Execs     170000 -    14031.6/s
Execs     180000 -    14030.2/s
Execs     190000 -    14076.4/s
Execs     200000 -    14016.3/s
Execs     210000 -    14010.8/s
```
