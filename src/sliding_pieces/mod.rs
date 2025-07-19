use crate::{bitboard::BitBoard, consts::Square, magics::MagicEntry, precomputed::{BISHOP_MAGICS, BISHOP_MOVES, ROOK_MAGICS, ROOK_MOVES}};

fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers.0 & entry.mask;
    let hash = blockers.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset as usize + index
}

pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &ROOK_MAGICS[square as usize];
    BitBoard(ROOK_MOVES[magic_index(magic, blockers)])
}

pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &BISHOP_MAGICS[square as usize];
    BitBoard(BISHOP_MOVES[magic_index(magic, blockers)])
}

pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let rook_moves = get_rook_moves(square, blockers);
    let bishop_moves = get_bishop_moves(square, blockers);
    rook_moves | bishop_moves
}