use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct XorShiro128ss {
    state_x: usize,
    state_y: usize,
}

impl GeneratorTrait for XorShiro128ss {
    #[inline]
    fn rand(&mut self) -> usize {
        let s0 = self.state_x;
        let mut s1 = self.state_y;
        let res = s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        s1 ^= s0;
        self.state_x = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
        self.state_y = s1.rotate_left(37);
        res
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 2);
        self.state_x = seeds.state_w;
        self.state_y = seeds.state_x;
    }
}

impl XorShiro128ss {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 2);
        Self {
            state_x: seeds.state_w,
            state_y: seeds.state_x,
        }
    }
}
