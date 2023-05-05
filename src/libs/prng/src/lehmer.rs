use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct Lehmer64 {
    state: u128,
}

impl GeneratorTrait for Lehmer64 {
    #[inline]
    fn rand(&mut self) -> usize {
        self.state = self.state.wrapping_mul(0xda94_2042_e4dd_58b5);
        (self.state >> 64) as usize
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 1);
        self.state = u128::from(seeds.state_w as u64);
    }
}

impl Lehmer64 {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 1);
        Self {
            state: u128::from(seeds.state_w as u64),
        }
    }
}
