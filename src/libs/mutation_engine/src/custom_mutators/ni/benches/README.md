# Benchmark

Tests were conducted on a MacBook Pro M1 Pro

```shell
``running 12 tests
test bench_original_ni_area_100         ... bench:       3,183 ns/iter (+/- 158)
test bench_original_ni_area_100k        ... bench:   5,678,027 ns/iter (+/- 1,693,456)
test bench_original_ni_area_10k         ... bench:     499,175 ns/iter (+/- 52,336)
test bench_original_ni_area_1k          ... bench:      42,827 ns/iter (+/- 5,332)
test bench_parallel_hybrid_ni_area_100  ... bench:       3,171 ns/iter (+/- 197)
test bench_parallel_hybrid_ni_area_100k ... bench:   2,726,152 ns/iter (+/- 11,234,681)
test bench_parallel_hybrid_ni_area_10k  ... bench:     368,689 ns/iter (+/- 26,843)
test bench_parallel_hybrid_ni_area_1k   ... bench:      31,727 ns/iter (+/- 4,371)
test bench_parallel_ni_area_100         ... bench:       3,122 ns/iter (+/- 155)
test bench_parallel_ni_area_100k        ... bench:     332,651 ns/iter (+/- 50,611)
test bench_parallel_ni_area_10k         ... bench:     658,863 ns/iter (+/- 208,068)
test bench_parallel_ni_area_1k          ... bench:      31,543 ns/iter (+/- 5,698)

test result: ok. 0 passed; 0 failed; 0 ignored; 12 measured; 0 filtered out; finished in 57.85s`
```

Based on these micro benchmarks here it looks like the recursive parallel version
is the fastest by a large margin. In the execution of the overall mutation framework
it showed that the hybrid version seems to be faster over the original implementation
by factor 2 and over the recursive parallel version by a factor of ~1.5.
