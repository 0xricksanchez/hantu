use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

const fn rol64(x: u64, k: i32) -> u64 {
    (x << k) | (x >> (64 - k))
}

#[derive(Debug, Clone, Copy)]
pub struct XorShiro256ss {
    state_w: usize,
    state_x: usize,
    state_y: usize,
    state_z: usize,
}

impl GeneratorTrait for XorShiro256ss {
    #[inline]
    fn rand(&mut self) -> usize {
        let res = rol64(self.state_x.wrapping_mul(5) as u64, 7).wrapping_mul(9) as usize;
        let t = self.state_y.wrapping_shl(17);
        self.state_y ^= self.state_w;
        self.state_z ^= self.state_x;
        self.state_x ^= self.state_y;
        self.state_w ^= self.state_z;
        self.state_y ^= t;
        self.state_z = rol64(self.state_z as u64, 45) as usize;
        res
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 4);
        self.state_w = seeds.state_w;
        self.state_x = seeds.state_x;
        self.state_y = seeds.state_y;
        self.state_z = seeds.state_z;
    }
}

impl XorShiro256ss {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 4);
        Self {
            state_w: seeds.state_w,
            state_x: seeds.state_x,
            state_y: seeds.state_y,
            state_z: seeds.state_z,
        }
    }
}
