use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct Xorshift64 {
    state: usize,
}

impl GeneratorTrait for Xorshift64 {
    #[inline]
    fn rand(&mut self) -> usize {
        let val = self.state;
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 43;
        val
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 1);
        self.state = seeds.state_w;
    }
}

impl Xorshift64 {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 1);
        Self {
            state: seeds.state_w,
        }
    }
}
