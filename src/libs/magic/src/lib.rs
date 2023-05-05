pub const MAGIC_8: [u8; 27] = [
    0x7f, 0xff, 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
    0x10, 0x20, 0x30, 0x40, 0x7e, 0x80, 0x81, 0xc0, 0xfe,
];
pub const MAGIC_16: [u16; 63] = [
    0x7fff, 0xffff, 0x0, 0x0101, 0x8080, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb,
    0xc, 0xd, 0xe, 0xf, 0x10, 0x20, 0x40, 0x7e, 0x7f, 0x80, 0x81, 0xc0, 0xfe, 0xff, 0x7eff, 0x8000,
    0x8001, 0xfffe, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700, 0x800, 0x900, 0xa00, 0xb00,
    0xc00, 0xd00, 0xe00, 0xf00, 0x1000, 0x2000, 0x4000, 0x7e00, 0x7f00, 0x8000, 0x8100, 0xc000,
    0xfe00, 0xff00, 0xff7e, 0xff7f, 0x0180, 0xfeff,
];
pub const MAGIC_32: [u32; 63] = [
    0x0,
    0x1,
    0x2,
    0x3,
    0x4,
    0x5,
    0x6,
    0x7,
    0x8,
    0x9,
    0xa,
    0xb,
    0xc,
    0xd,
    0xe,
    0xf,
    0x10,
    0x20,
    0x40,
    0x7e,
    0x7f,
    0x80,
    0x81,
    0xc0,
    0xfe,
    0xff,
    0x7ffff,
    0x0100_0000,
    0x0200_0000,
    0x0300_0000,
    0x0400_0000,
    0x0500_0000,
    0x0600_0000,
    0x0700_0000,
    0x0800_0000,
    0x0900_0000,
    0x0a00_0000,
    0x0b00_0000,
    0x0c00_0000,
    0x0d00_0000,
    0x0e00_0000,
    0x0f00_0000,
    0x8000_0000,
    0x4000_0000,
    0xffff_ffff,
    0x0101_0101,
    0x8080_8080,
    0x7eff_ffff,
    0x8000_0021,
    0xffff_fffe,
    0x1000_0000,
    0x2000_0000,
    0x4000_0000,
    0x7e00_0000,
    0x7f00_0000,
    0x8100_0000,
    0xc000_0000,
    0xfe00_0000,
    0xff00_0000,
    0xffff_ff7e,
    0xfff_fff7f,
    0x0100_0080,
    0xfeff_ffff,
];
pub const MAGIC_64: [u64; 61] = [
    0xffff_ffff_ffff_ffff,
    0x4000_0000_0000_0000,
    0x8000_0000_0000_0000,
    0x7fff_ffff_ffff_ffff,
    0x0,
    0x0101_0101_0101_0101,
    0x8080_8080_8080_8080,
    0x1,
    0x2,
    0x3,
    0x4,
    0x5,
    0x6,
    0x7,
    0x8,
    0x9,
    0xa,
    0xb,
    0xc,
    0xd,
    0xe,
    0xf,
    0x10,
    0x20,
    0x40,
    0x7e,
    0x7f,
    0x80,
    0x81,
    0xc0,
    0xfe,
    0xff,
    0x7f,
    0x7eff_ffff_ffff_ffff,
    0x8000_0000_0000_0001,
    0xffff_ffff_ffff_fffe,
    0x0100_0000_0000_0000,
    0x0200_0000_0000_0000,
    0x0300_0000_0000_0000,
    0x0400_0000_0000_0000,
    0x0500_0000_0000_0000,
    0x0600_0000_0000_0000,
    0x0700_0000_0000_0000,
    0x0900_0000_0000_0000,
    0x0a00_0000_0000_0000,
    0x0b00_0000_0000_0000,
    0x0c00_0000_0000_0000,
    0x0d00_0000_0000_0000,
    0x0e00_0000_0000_0000,
    0x0f00_0000_0000_0000,
    0x1000_0000_0000_0000,
    0x2000_0000_0000_0000,
    0x4000_0000_0000_0000,
    0x7e00_0000_0000_0000,
    0x7f00_0000_0000_0000,
    0x8100_0000_0000_0000,
    0xc000_0000_0000_0000,
    0xfe00_0000_0000_0000,
    0xff00_0000_0000_0000,
    0x0100_0000_0000_0080,
    0xfeff_ffff_ffff_ffff,
];
