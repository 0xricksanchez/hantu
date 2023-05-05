use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct Wyhash64 {
    state: usize,
}

impl GeneratorTrait for Wyhash64 {
    #[inline]
    fn rand(&mut self) -> usize {
        self.state = self.state.wrapping_add(0x60be_e2be_e120_fc15);
        let mut tmp = u128::from(self.state as u64).wrapping_mul(0xa3b1_9535_4a39_b70d);
        let m1 = (tmp >> 64) ^ tmp;
        tmp = m1.wrapping_mul(0x1b03_7387_12fa_d5c9);
        ((tmp >> 64) ^ tmp) as usize
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 1);
        self.state = seeds.state_w;
    }
}

impl Wyhash64 {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 1);
        Self {
            state: seeds.state_w,
        }
    }
}
