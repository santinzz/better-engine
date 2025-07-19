use rand::{rngs::ThreadRng, RngCore};

use crate::{bitboard::BitBoard, board::Piece, consts::Square};
use std::time::Instant;

pub const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (0, -1), (-1, 0), (0, 1)];
pub const BISHOP_DELTAS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

struct MagicEntry {
    mask: BitBoard,
    magic: u64,
    shift: u8,
}

fn relevant_blockers(piece: Piece, square: Square) -> BitBoard {
    let deltas = match piece {
        Piece::Rook => &ROOK_DELTAS,
        Piece::Bishop => &BISHOP_DELTAS,
        _ => unreachable!(),
    };

    let mut blockers = BitBoard::EMPTY;

    for &(df, dr) in deltas {
        let mut ray = square;
        while let Some(shifed) = ray.try_offset(df, dr) {
            blockers |= ray.bb();
            ray = shifed;
        }
    }

    blockers &= !square.bb();
    blockers
}

fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    index
}

// Given a sliding piece and a square, finds a magic number that
// perfectly maps input blockers into its solution in a hash table
fn find_magic(
    piece: Piece,
    square: Square,
    index_bits: u8,
    rng: &mut ThreadRng,
) -> (MagicEntry, Vec<BitBoard>) {
    let mask = relevant_blockers(piece, square);
    let shift = 64 - index_bits;
    loop {
        let magic = rng.next_u64() & rng.next_u64() & rng.next_u64();
        let magic_entry = MagicEntry { mask, magic, shift };
        if let Ok(table) = try_make_table(piece, square, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

struct TableFillError;

fn try_make_table(
    piece: Piece,
    square: Square,
    magic_entry: &MagicEntry,
) -> Result<Vec<BitBoard>, TableFillError> {
    let index_bits = 64 - magic_entry.shift;
    let mut table = vec![BitBoard::EMPTY; 1 << index_bits];

    let mut blockers = BitBoard::EMPTY;
    loop {
        let moves = match piece {
            Piece::Rook => piece.sliding_moves(square, blockers),
            Piece::Bishop => piece.sliding_moves(square, blockers),
            _ => unreachable!(),
        };

        let table_entry = &mut table[magic_index(magic_entry, blockers)];
        if table_entry.is_empty() {
            *table_entry = moves;
        } else if *table_entry != moves {
            return Err(TableFillError);
        }

        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers.is_empty() {
            break;
        }
    }
    Ok(table)
}

fn find_and_print_all_magics(sliding_piece: Piece, rng: &mut ThreadRng) {
    println!(
        "pub const {}_MAGICS: &[MagicEntry; Square::NUM] = &[",
        sliding_piece.name()
    );
    let mut total_table_size = 0;
    for &square in &Square::ALL {
        let index_bits = relevant_blockers(sliding_piece, square).popcnt() as u8;
        let (entry, table) = find_magic(sliding_piece, square, index_bits, rng);
        // In the final move generator, each table is concatenated into one contiguous table
        // for convenience, so an offset is added to denote the start of each segment.
        println!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask.0, entry.magic, entry.shift, total_table_size
        );
        total_table_size += table.len();
    }
    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usize = {};",
        sliding_piece.name().to_uppercase(),
        total_table_size
    );
}

// #[cfg(test)]
// mod tests {
//     use std::time::Instant;

//     use rand::prelude::*;

//     use crate::{board::Piece, magic_gen::find_and_print_all_magics};

//     #[test]
//     fn test_find_and_print_all_magics() {
//         let intstant = Instant::now();
//         let mut rng = rand::rng();
//         find_and_print_all_magics(Piece::Rook, &mut rng);
//         find_and_print_all_magics(Piece::Bishop, &mut rng);
//         println!("Time taken: {:?}", intstant.elapsed());
//     }
// }