use crate::{bitboard::BitBoard, board::Board, game_result::GameResult, moves::Flags};
use std::cell::RefCell;
use crate::moves::Move;

thread_local! {
    static MOVE_BUF: RefCell<Vec<Move>> = RefCell::new(Vec::with_capacity(128));
}

pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::with_capacity(256);
    board.generate_legal_moves_into(&mut moves);
    // 2) Iterate over the clone, recursing
    let mut nodes = 0;
    for mv in moves {
        let mut board_copy = board.clone();
        board_copy.make_move(&mv);
        nodes += perft(&board_copy, depth - 1);
    }
    nodes
}

#[cfg(test)]
mod tests {
    use crate::perft::{perft};

    #[test]
    fn test_perft() {
        let mut board = crate::Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(perft(&board, 6), 119060324);
    }
}
