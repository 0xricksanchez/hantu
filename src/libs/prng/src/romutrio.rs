use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;

#[derive(Debug, Clone, Copy)]
pub struct RomuTrio {
    pub state_x: usize,
    pub state_y: usize,
    pub state_z: usize,
}

impl GeneratorTrait for RomuTrio {
    #[inline]
    fn rand(&mut self) -> usize {
        let xp = self.state_x;
        let yp = self.state_y;
        let zp = self.state_z;
        self.state_x = 15_241_094_284_759_029_579_usize.wrapping_mul(zp);
        self.state_y = yp.wrapping_sub(xp).rotate_left(12);
        self.state_z = zp.wrapping_sub(yp).rotate_left(44);
        xp
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 3);
        self.state_x = seeds.state_w;
        self.state_y = seeds.state_x;
        self.state_z = seeds.state_y;
    }
}

impl RomuTrio {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 3);
        Self {
            state_x: seeds.state_w,
            state_y: seeds.state_x,
            state_z: seeds.state_y,
        }
    }
}
