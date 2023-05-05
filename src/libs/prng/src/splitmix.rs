use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct SplitMix64 {
    state: usize,
}

impl GeneratorTrait for SplitMix64 {
    #[inline]
    fn rand(&mut self) -> usize {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut x = self.state;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 1);
        self.state = seeds.state_w;
    }
}

impl SplitMix64 {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 1);
        Self {
            state: seeds.state_w,
        }
    }
}
