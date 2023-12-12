#[derive(Debug)]
pub struct Seeds {
    pub state_w: usize,
    pub state_x: usize,
    pub state_y: usize,
    pub state_z: usize,
}

#[macro_export]
macro_rules! get_seeds {
    ($seed:expr, $num:expr) => {{
        use $crate::ENTROPY;
        

        #[cfg(target_arch = "x86_64")]
        pub fn get_rdtsc() -> usize {
            unsafe { std::arch::x86_64::_rdtsc() as usize }
        }

        // https://lore.kernel.org/lkml/20200914115311.2201-3-leo.yan@linaro.org/
        #[cfg(target_arch = "aarch64")]
        pub fn get_rdtsc() -> usize {
            let mut ctr: u64 = 0;
            unsafe {
                asm!("mrs {x0}, cntvct_el0", x0 = inout(reg) ctr);
            }
            return ctr as usize;
        }

        fn generate_seeds(init_seed: usize, num_seeds: usize) -> Vec<usize> {
            let mut last_seed = if init_seed == 0 {
                get_rdtsc() ^ 0xdeadbeefcafebabe
            } else {
                init_seed
            };

            assert!(
                (1..=4).contains(&num_seeds),
                "num_seeds must be between 1 and 4 (inclusive)"
            );

            let mut seeds = vec![0; num_seeds];
            for i in 0..num_seeds {
                let new_seed = last_seed ^ ENTROPY ^ i;
                seeds[i] = new_seed;
                last_seed = new_seed;
            }

            seeds
        }

        let seeds = generate_seeds($seed, $num);
        Seeds {
            state_w: *seeds.get(0).unwrap_or(&0),
            state_x: *seeds.get(1).unwrap_or(&0),
            state_y: *seeds.get(2).unwrap_or(&0),
            state_z: *seeds.get(3).unwrap_or(&0),
        }
    }};
}
