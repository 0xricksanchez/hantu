use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct RomuDuoJr {
    state_x: usize,
    state_y: usize,
}

impl GeneratorTrait for RomuDuoJr {
    #[inline]
    fn rand(&mut self) -> usize {
        let xp = self.state_x;
        self.state_x = 15_241_094_284_759_029_579_usize.wrapping_mul(self.state_y);
        self.state_y = self.state_y.wrapping_sub(xp).rotate_left(27);
        xp
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 2);
        self.state_x = seeds.state_w;
        self.state_y = seeds.state_x;
    }
}

impl RomuDuoJr {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 2);
        Self {
            state_x: seeds.state_w,
            state_y: seeds.state_x,
        }
    }
}
