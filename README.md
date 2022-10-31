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
Avergae test case has 55782 bytes
Execs 10000 -    14745.7/s - 51240
Execs 20000 -    14259.2/s - 34864
Execs 30000 -    14159.4/s - 43016
Execs 40000 -    14139.4/s - 63416
Execs 50000 -    14104.9/s - 73815
Execs 60000 -    14211.4/s - 55344
Execs 70000 -    14179.2/s - 55392
Execs 80000 -    14149.3/s - 34792
Execs 90000 -    14229.5/s - 43024
Execs 100000 -    14297.8/s - 42920
Execs 110000 -    14376.1/s - 48567
```
