use std::sync::OnceLock;

use crate::bitboard::BitBoard;

pub const KNIGHT_MOVES: [i8; 8] = [-17, -15, -10, -6, 6, 10, 15, 17];
pub const KING_MOVES: [i8; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];

macro_rules! simple_enum {
    ($(
        pub enum $name:ident {
            $($variant:ident),*
        }
    )*) => {$(
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const NUM: usize = [$(Self::$variant),*].len();
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];

            pub fn try_index(index: usize) -> Option<Self> {
                $(#[allow(non_upper_case_globals, unused)]
                const $variant: usize = $name::$variant as usize;)*
                #[allow(non_upper_case_globals)]
                match index {
                    $($variant => Option::Some(Self::$variant),)*
                    _ => Option::None
                }
            }

            pub fn index(index: usize) -> Self {
                Self::try_index(index).unwrap_or_else(|| panic!("Index {} is out of range.", index))
            }
        }
    )*};
}

simple_enum! {
    pub enum File {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H
    }

    pub enum Rank {
        First,
        Second,
        Third,
        Fourth,
        Fifth,
        Sixth,
        Seventh,
        Eighth
    }

    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8
    }
}

impl Square {
    pub fn from_index(index: u8) -> Square {
        unsafe {
            std::mem::transmute::<u8, Square>(index)
        }
    }

    pub fn new(file: File, rank: Rank) -> Option<Square> {
        Some(Square::index((rank as usize) << 3 | (file as usize)))
    }

    pub fn file(self) -> File {
        File::index(self as usize & 0b000111)
    }

    pub fn rank(self) -> Rank {
        Rank::index(self as usize >> 3)
    }

    pub fn bb(self) -> BitBoard {
        BitBoard(1u64 << (self as usize))
    }

    pub fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        Square::new(
            File::try_index((self.file() as i8 + file_offset).try_into().ok()?)?,
            Rank::try_index((self.rank() as i8 + rank_offset).try_into().ok()?)?,
        )
    }
}

pub struct Magic {
    pub magic: u64,
    pub mask: u64,
    pub shift: u32,
    pub offset: usize
}

pub const DIRECTION_OFFSETS: [i32; 8] = [8, -8, -1, 1, 7, -7, 9, -9];


// --- Magic Bitboard Constants for Sliding Pieces ---
// These values are typically found by a separate precomputation program.
// They are hardcoded here for demonstration.

// Magic numbers for Rook attacks for each square (0-63)

// Number of relevant occupancy bits for Rook attacks for each square
pub const ROOK_SHIFTS: [u8; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    52, 53, 53, 53, 53, 53, 53, 52,
];

// Magic numbers for Bishop attacks for each square (0-63)
// Number of relevant occupancy bits for Bishop attacks for each square
pub const BISHOP_SHIFTS: [u8; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58,
    59, 60, 60, 60, 60, 60, 60, 59,
    59, 60, 60, 60, 60, 60, 60, 59,
    59, 60, 60, 60, 60, 60, 60, 59,
    59, 60, 60, 60, 60, 60, 60, 59,
    59, 60, 60, 60, 60, 60, 60, 59,
    59, 60, 60, 60, 60, 60, 60, 59,
    58, 59, 59, 59, 59, 59, 59, 58,
];

// Masks for relevant occupancy bits for Rook attacks
// These are the squares that can potentially block a rook's attack from a given square.
pub static ROOK_MASKS: OnceLock<[u64; 64]> = OnceLock::new();
// Masks for relevant occupancy bits for Bishop attacks
pub static BISHOP_MASKS: OnceLock<[u64; 64]> = OnceLock::new();

// Precomputed attack tables for Rooks and Bishops.
// These will be populated once at startup.
// The size of these arrays depends on the number of relevant occupancy bits for each square.
// For example, a rook on A1 has 12 relevant occupancy bits (6 on rank 1, 6 on file A, excluding A1 itself).
// So, it would have 2^12 = 4096 possible attack patterns.
// The total size is sum(2^(64 - shift)) for all squares.
// pub static ROOK_ATTACKS: OnceLock<Vec<u64>> = OnceLock::new();
// pub static BISHOP_ATTACKS: OnceLock<Vec<u64>> = OnceLock::new();

// Offsets into the combined ROOK_ATTACKS and BISHOP_ATTACKS vectors
// This allows us to store all attack tables in a single vector and use an offset + index.
pub static ROOK_OFFSETS: OnceLock<[usize; 64]> = OnceLock::new();
pub static BISHOP_OFFSETS: OnceLock<[usize; 64]> = OnceLock::new();

pub const KNIGHT_ATTACKS: [u64; 64] = [
    0x20400,
    0x50800,
    0xa1100,
    0x142200,
    0x284400,
    0x508800,
    0xa01000,
    0x402000,
    0x2040004,
    0x5080008,
    0xa110011,
    0x14220022,
    0x28440044,
    0x50880088,
    0xa0100010,
    0x40200020,
    0x204000402,
    0x508000805,
    0xa1100110a,
    0x1422002214,
    0x2844004428,
    0x5088008850,
    0xa0100010a0,
    0x4020002040,
    0x20400040200,
    0x50800080500,
    0xa1100110a00,
    0x142200221400,
    0x284400442800,
    0x508800885000,
    0xa0100010a000,
    0x402000204000,
    0x2040004020000,
    0x5080008050000,
    0xa1100110a0000,
    0x14220022140000,
    0x28440044280000,
    0x50880088500000,
    0xa0100010a00000,
    0x40200020400000,
    0x204000402000000,
    0x508000805000000,
    0xa1100110a000000,
    0x1422002214000000,
    0x2844004428000000,
    0x5088008850000000,
    0xa0100010a0000000,
    0x4020002040000000,
    0x400040200000000,
    0x800080500000000,
    0x1100110a00000000,
    0x2200221400000000,
    0x4400442800000000,
    0x8800885000000000,
    0x100010a000000000,
    0x2000204000000000,
    0x4020000000000,
    0x8050000000000,
    0x110a0000000000,
    0x22140000000000,
    0x44280000000000,
    0x88500000000000,
    0x10a000000000000,
    0x20400000000000,
];

const fn init_pawn_attack_masks() -> [[u64; 64]; 2] {
    let mut masks = [[0u64; 64]; 2];
    let mut square = 0;

    while square < 64 {
        let rank = square / 8;
        let file = square % 8;

        // White pawn attacks (index 0)
        if rank < 7 {
            if file > 0 {
                masks[0][square] |= 1u64 << ((rank + 1) * 8 + (file - 1));
            }
            if file < 7 {
                masks[0][square] |= 1u64 << ((rank + 1) * 8 + (file + 1));
            }
        }

        // Black pawn attacks (index 1)
        if rank > 0 {
            if file > 0 {
                masks[1][square] |= 1u64 << ((rank - 1) * 8 + (file - 1));
            }
            if file < 7 {
                masks[1][square] |= 1u64 << ((rank - 1) * 8 + (file + 1));
            }
        }

        square += 1;
    }

    masks
}

pub const PAWN_ATTACKS: [[u64; 64]; 2] = init_pawn_attack_masks();

const fn init_sliding_attack_masks() -> [[u64; 64]; 2] {
    let mut masks = [[0u64; 64]; 2];
    let mut square = 0;

    while square < 64 {
        let rank = square / 8;
        let file = square % 8;

        // Rook directions (index 0)
        {
            let mut r = rank + 1;
            while r < 8 {
                let target = r * 8 + file;
                masks[0][square] |= 1u64 << target;
                r += 1;
            }

            let mut r = rank;
            while r > 0 {
                r -= 1;
                let target = r * 8 + file;
                masks[0][square] |= 1u64 << target;
            }

            let mut f = file + 1;
            while f < 8 {
                let target = rank * 8 + f;
                masks[0][square] |= 1u64 << target;
                f += 1;
            }

            let mut f = file;
            while f > 0 {
                f -= 1;
                let target = rank * 8 + f;
                masks[0][square] |= 1u64 << target;
            }
        }

        // Bishop directions (index 1)
        {
            let mut r = rank + 1;
            let mut f = file + 1;
            while r < 8 && f < 8 {
                masks[1][square] |= 1u64 << (r * 8 + f);
                r += 1;
                f += 1;
            }

            let mut r = rank + 1;
            let mut f = file;
            while r < 8 && f > 0 {
                f -= 1;
                masks[1][square] |= 1u64 << (r * 8 + f);
                r += 1;
            }

            let mut r = rank;
            let mut f = file + 1;
            while r > 0 && f < 8 {
                r -= 1;
                masks[1][square] |= 1u64 << (r * 8 + f);
                f += 1;
            }

            let mut r = rank;
            let mut f = file;
            while r > 0 && f > 0 {
                r -= 1;
                f -= 1;
                masks[1][square] |= 1u64 << (r * 8 + f);
            }
        }

        square += 1;
    }

    masks
}

const SLIDING_ATTACKS: [[u64; 64]; 2] = init_sliding_attack_masks();

pub const ROOK_ATTACKS: [u64; 64] = SLIDING_ATTACKS[0];
pub const BISHOP_ATTACKS: [u64; 64] = SLIDING_ATTACKS[1];

const NORTH: usize = 0;
const SOUTH: usize = 1;
const EAST: usize = 2;
const WEST: usize = 3;
const NORTHEAST: usize = 4;
const SOUTHWEST: usize = 5;
const NORTHWEST: usize = 6;
const SOUTHEAST: usize = 7;


pub const W_KINGSIDE_RIGHTS: u8 = 0b0001;
pub const W_QUEENSIDE_RIGHTS: u8 = 0b0010;
pub const B_KINGSIDE_RIGHTS: u8 = 0b0100;
pub const B_QUEENSIDE_RIGHTS: u8 = 0b1000;

pub const KING_ATTACKS: [BitBoard; 64] = generate_king_attacks();

const fn generate_king_attacks() -> [BitBoard; 64] {
    let mut attacks = [BitBoard::EMPTY; 64];
    // All 8 directions a king can move
    let directions: [(i8, i8); 8] = [
        (-1, -1), ( 0, -1), ( 1, -1),
        (-1,  0),          ( 1,  0),
        (-1,  1), ( 0,  1), ( 1,  1),
    ];

    let mut sq = 0;
    while sq < 64 {
        let rank = (sq / 8) as i8;
        let file = (sq % 8) as i8;
        let mut mask = 0u64;

        // for each direction, see if the target square is on board
        let mut i = 0;
        while i < 8 {
            let (df, dr) = directions[i];
            let nf = file + df;
            let nr = rank + dr;
            if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                let nsq = (nr * 8 + nf) as usize;
                mask |= 1u64 << nsq;
            }
            i += 1;
        }

        attacks[sq] = BitBoard(mask);
        sq += 1;
    }

    attacks
}