use crate::get_seeds;
use crate::seed::Seeds;
use crate::GeneratorTrait;
use packed_simd_2::{u32x8, u64x4, IntoBits};

const PHI: [u64; 16] = [
    0x9E37_79B9_7F4A_7C15,
    0xF39C_C060_5CED_C834,
    0x1082_276B_F3A2_7251,
    0xF86C_6A11_D0C1_8E95,
    0x2767_F0B1_53D2_7B7F,
    0x0347_045B_5BF1_827F,
    0x0188_6F09_2840_3002,
    0xC1D6_4BA4_0F33_5E36,
    0xF06A_D7AE_9717_877E,
    0x8583_9D6E_FFBD_7DC6,
    0x64D3_25D1_C537_1682,
    0xCADD_0CCC_FDFF_BBE1,
    0x626E_33B8_D04B_4331,
    0xBBF7_3C79_0D94_F79D,
    0x471C_4AB3_ED3D_82A5,
    0xFEC5_0770_5E4A_E6E5,
];

pub const STATE_LANES: usize = u64x4::lanes();
const STATE_SIZE: usize = 4;
// Original values from the blog post
const STEPS: usize = 5;
const ROUNDS: usize = 4;

/// Implements <https://espadrine.github.io/blog/posts/shishua-the-fastest-prng-in-the-world.html>
/// Adapted from <https://github.com/dbartussek/shishua_rs>
#[derive(Debug, Clone, Copy)]
pub struct ShiShua {
    state: [u64x4; STATE_SIZE],
    output: [u64x4; STATE_SIZE],
    counter: u64x4,
    buffer_idx: usize,
    arr_idx: usize,
}

impl ShiShua {
    pub fn new(seed: usize) -> Self {
        let seeds: Seeds = get_seeds!(seed, 4);
        let mut buffer = [0_u64; STATE_LANES * STATE_SIZE * ROUNDS];

        let mut state = Self {
            state: [
                u64x4::new(
                    PHI[3],
                    PHI[2] ^ seeds.state_x as u64,
                    PHI[1],
                    PHI[0] ^ seeds.state_w as u64,
                ),
                u64x4::new(
                    PHI[7],
                    PHI[6] ^ seeds.state_z as u64,
                    PHI[5],
                    PHI[4] ^ seeds.state_y as u64,
                ),
                u64x4::new(
                    PHI[11],
                    PHI[10] ^ seeds.state_z as u64,
                    PHI[9],
                    PHI[8] ^ seeds.state_y as u64,
                ),
                u64x4::new(
                    PHI[15],
                    PHI[14] ^ seeds.state_x as u64,
                    PHI[13],
                    PHI[12] ^ seeds.state_w as u64,
                ),
            ],
            output: [u64x4::splat(0); 4],
            counter: u64x4::splat(0),
            buffer_idx: 0,
            arr_idx: 0,
        };

        for _ in 0..STEPS {
            state.generate(&mut buffer);
            state.state[0] = state.output[3];
            state.state[1] = state.output[2];
            state.state[2] = state.output[1];
            state.state[3] = state.output[0];
        }

        state
    }

    fn generate(&mut self, output_slice: &mut [u64]) {
        assert_eq!(output_slice.len() % (STATE_LANES * STATE_SIZE), 0);
        for output_chunk in output_slice.chunks_exact_mut(STATE_LANES * STATE_SIZE) {
            let output = self.round_unpack();
            output_chunk.copy_from_slice(&output);
        }
    }

    pub fn round_unpack(&mut self) -> [u64; STATE_SIZE * STATE_LANES] {
        let raw = self.round();
        let mut output = [0_u64; STATE_SIZE * STATE_LANES];

        for (group, value) in raw.iter().enumerate() {
            let group_slice_index = group * STATE_LANES;
            for i in 0..STATE_LANES {
                output[group_slice_index + i] = value.extract(STATE_LANES - 1 - i);
            }
        }
        output
    }

    #[inline(always)]
    fn round(&mut self) -> [u64x4; STATE_SIZE] {
        const fn correct_index(index: u32) -> u32 {
            (u32x8::lanes() as u32 - 1 - index) ^ 1
        }

        // Shuffle values work differently in Rust than in the C source.
        //
        // High and low 32 bits are flipped.
        // Indexing is the other way around
        let shuffle = [
            // u32x8::new(4, 3, 2, 1, 0, 7, 6, 5),
            u32x8::new(
                correct_index(3),
                correct_index(4),
                correct_index(1),
                correct_index(2),
                correct_index(7),
                correct_index(0),
                correct_index(5),
                correct_index(6),
            ),
            // u32x8::new(2, 1, 0, 7, 6, 5, 4, 3),
            u32x8::new(
                correct_index(1),
                correct_index(2),
                correct_index(7),
                correct_index(0),
                correct_index(5),
                correct_index(6),
                correct_index(3),
                correct_index(4),
            ),
        ];

        let increment = u64x4::new(1, 3, 5, 7);

        let Self {
            state,
            output,
            counter,
            ..
        } = self;

        // Perform the round
        state[1] += *counter;
        state[3] += *counter;
        *counter += increment;

        let u0 = state[0] >> 1;
        let u1 = state[1] >> 3;
        let u2 = state[2] >> 1;
        let u3 = state[3] >> 3;

        fn shuffle_u64_as_u32(state: u64x4, shuffle: u32x8) -> u64x4 {
            let state_u32: u32x8 = state.into_bits();
            let shuffled = state_u32.shuffle1_dyn(shuffle);

            shuffled.into_bits()
        }

        let t0 = shuffle_u64_as_u32(state[0], shuffle[0]);
        let t1 = shuffle_u64_as_u32(state[1], shuffle[1]);
        let t2 = shuffle_u64_as_u32(state[2], shuffle[0]);
        let t3 = shuffle_u64_as_u32(state[3], shuffle[1]);

        state[0] = t0 + u0;
        state[1] = t1 + u1;
        state[2] = t2 + u2;
        state[3] = t3 + u3;

        let result = *output;

        output[0] = u0 ^ t1;
        output[1] = u2 ^ t3;
        output[2] = state[0] ^ state[3];
        output[3] = state[2] ^ state[1];

        result
    }
}

impl GeneratorTrait for ShiShua {
    #[inline]
    fn rand(&mut self) -> usize {
        // If we finish reading from 1 lane we get to the next
        if self.buffer_idx % 3 == 0 {
            self.buffer_idx = 0;
            self.arr_idx += 1;
        }
        // If we consumed all lanes and indices, we roll new values.
        if self.arr_idx % 3 == 0 && self.buffer_idx % 3 == 0 {
            self.buffer_idx = 0;
            self.arr_idx = 0;
            self.round_unpack();
        }
        let out = self.output[self.arr_idx].extract(self.buffer_idx) as usize;
        self.buffer_idx += 1;

        out
    }

    fn set_seed(&mut self, seed: usize) {
        let seeds: Seeds = get_seeds!(seed, 4);
        self.state = [
            u64x4::new(
                PHI[3],
                PHI[2] ^ seeds.state_x as u64,
                PHI[1],
                PHI[0] ^ seeds.state_w as u64,
            ),
            u64x4::new(
                PHI[7],
                PHI[6] ^ seeds.state_z as u64,
                PHI[5],
                PHI[4] ^ seeds.state_y as u64,
            ),
            u64x4::new(
                PHI[11],
                PHI[10] ^ seeds.state_z as u64,
                PHI[9],
                PHI[8] ^ seeds.state_y as u64,
            ),
            u64x4::new(
                PHI[15],
                PHI[14] ^ seeds.state_x as u64,
                PHI[13],
                PHI[12] ^ seeds.state_w as u64,
            ),
        ];
    }
}
