use crate::{board::Piece, consts::Square};

const fn init_sliding_attack_masks() -> [[u64; 64]; 2] {
    let mut masks = [[0u64; 64]; 2];
    let mut square = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;

        // North
        let mut r = rank + 1;
        while r < 8 {
            masks[0][square] |= 1u64 << (square + (r - rank) * 8);
            r += 1;
        }

        // South
        let mut r = rank.wrapping_sub(1);
        while r < 8 {
            masks[0][square] |= 1u64 << (square.wrapping_sub((rank - r) * 8));
            r = r.wrapping_sub(1);
        }

        // East
        let mut f = file + 1;
        while f < 8 {
            masks[0][square] |= 1u64 << (square + (f - file));
            f += 1;
        }

        // West
        let mut f = file.wrapping_sub(1);
        while f < 8 {
            masks[0][square] |= 1u64 << (square.wrapping_sub(file - f));
            f = f.wrapping_sub(1);
        }

        // Northeast
        let mut r = rank + 1;
        let mut f = file + 1;

        while r < 8 && f < 8 {
            masks[1][square] |= 1u64 << (square + (r - rank) * 8 + (f - file));
            r += 1;
            f += 1;
        }

        // Northwest
        let mut r = rank + 1;
        let mut f = file.wrapping_sub(1);
        while r < 8 && f < 8 {
            masks[1][square] |= 1u64 << (square + (r - rank) * 8 - (file - f));
            r += 1;
            f = f.wrapping_sub(1);
        }

        // Southeast
        let mut r = rank.wrapping_sub(1);
        let mut f = file + 1;
        while r < 8 && f < 8 {
            masks[1][square] |= 1u64 << (square.wrapping_sub((rank - r) * 8) + (f - file));
            r = r.wrapping_sub(1);
            f += 1;
        }

        // Southwest
        let mut r = rank.wrapping_sub(1);
        let mut f = file.wrapping_sub(1);
        while r < 8 && f < 8 {
            masks[1][square] |= 1u64 << (square.wrapping_sub((rank - r) * 8) + (file - f));
            r = r.wrapping_sub(1);
            f = f.wrapping_sub(1);
        }

        square += 1;
     }

    masks
}

const SLIDING_ATTACK_MASKS: [[u64; 64]; 2] = init_sliding_attack_masks();

pub const ROOK_ATTACK_MASKS: [u64; 64] = SLIDING_ATTACK_MASKS[0];
pub const BISHOP_ATTACK_MASKS: [u64; 64] = SLIDING_ATTACK_MASKS[1];

