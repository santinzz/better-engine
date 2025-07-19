use std::cell::RefCell;

use crate::{
    bitboard::BitBoard,
    board::{Board, Color, Piece},
    consts::{
        File, Rank, Square, B_KINGSIDE_RIGHTS, B_QUEENSIDE_RIGHTS, DIRECTION_OFFSETS, KING_ATTACKS,
        KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, W_KINGSIDE_RIGHTS, W_QUEENSIDE_RIGHTS,
    },
    precomputed::NumSquaresToTheEdge,
    sliding_pieces::{get_bishop_moves, get_queen_moves, get_rook_moves},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flags {
    Normal = 0x0,
    DoublePawnPush = 0x1,
    EnPassant = 0x2,
    Castling = 0x4,
    Capture = 0x10,
    Promotion = 0x20,
    PromotionCapture = 0x40,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub captured_piece: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: Flags, // e.g., 0x1 for double pawn push, 0x2 for en passant, 0x4 for castling
}

impl Board {
    pub fn generate_legal_moves_into(&self, moves: &mut Vec<Move>) {
        moves.clear();

        self.generate_pawn_moves(moves);
        self.generate_knight_moves(moves);
        self.generate_rook_moves(moves);
        self.generate_bishop_moves(moves);
        self.generate_queen_moves(moves);
        self.generate_king_moves(moves);

        moves.retain(|mv| {
            let mut board_copy = self.clone();
            board_copy.make_move(mv);
            !board_copy.is_king_in_check(self.turn.opposite())
        });
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        self.generate_pawn_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_king_moves(&mut moves);

        let mut legal_moves: Vec<Move> = Vec::new();

        for &mv in &moves {
            let mut board_copy = self.clone();

            board_copy.make_move(&mv);

            if !board_copy.is_king_in_check(self.turn.opposite()) {
                legal_moves.push(mv);
            }
        }

        legal_moves
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<Move>) {
        let mut our_pawns = if self.turn == Color::White {
            self.white_pawns
        } else {
            self.black_pawns
        };

        while our_pawns != BitBoard::EMPTY {
            let from_sq_idx = our_pawns.0.trailing_zeros() as u8;
            let rank = Square::from_index(from_sq_idx).rank();
            let file = Square::from_index(from_sq_idx).file();
            let pawn_color = self.turn;

            let (forward_dir, start_rank, promotion_rank, capture_dirs) = match pawn_color {
                Color::White => (8, Rank::Second, Rank::Eighth, [7, 9]),
                Color::Black => (-8, Rank::Seventh, Rank::First, [-7, -9]),
            };

            let (our_occupied, their_occupied) = match pawn_color {
                Color::White => (self.white_occupied, self.black_occupied),
                Color::Black => (self.black_occupied, self.white_occupied),
            };

            let target_sq_idx = (from_sq_idx as i8 + forward_dir) as u8;
            if target_sq_idx < 64 && (self.empty.0 & 1u64 << target_sq_idx) != 0 {
                if rank == promotion_rank {
                    for &promo_piece in &[Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(Move {
                            from: Square::from_index(from_sq_idx as u8),
                            to: Square::from_index(target_sq_idx as u8),
                            piece: Piece::Pawn,
                            promotion: Some(promo_piece),
                            captured_piece: None,
                            flags: Flags::Promotion,
                        });
                    }
                } else {
                    moves.push(Move {
                        from: Square::from_index(from_sq_idx as u8),
                        to: Square::from_index(target_sq_idx as u8),
                        piece: Piece::Pawn,
                        promotion: None,
                        captured_piece: None,
                        flags: Flags::Normal,
                    });
                }

                if rank == start_rank {
                    let double_target_sq_idx = (target_sq_idx as i8 + forward_dir) as u8;
                    if double_target_sq_idx < 64
                        && (self.empty.0 & 1u64 << double_target_sq_idx) != 0
                    {
                        moves.push(Move {
                            from: Square::from_index(from_sq_idx as u8),
                            to: Square::from_index(double_target_sq_idx as u8),
                            piece: Piece::Pawn,
                            promotion: None,
                            captured_piece: None,
                            flags: Flags::DoublePawnPush,
                        });
                    }
                }
            }

            for &capture_dir in &capture_dirs {
                let target_sq_idx = (from_sq_idx as i8 + capture_dir) as u8;
                let target_rank = target_sq_idx / 8;
                let target_file = target_sq_idx % 8;

                if target_sq_idx < 64
                    && (target_rank as i8 - rank as i8).abs() == 1
                    && (target_file as i8 - file as i8).abs() == 1
                {
                    let target_bit = BitBoard(1u64 << target_sq_idx);
                    if their_occupied.0 & target_bit.0 != 0 {
                        let captured_piece_type = self
                            .piece_on_square(Square::from_index(target_sq_idx as u8))
                            .map(|(p, _)| p);

                        if rank == promotion_rank {
                            for &promo_piece in
                                &[Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight]
                            {
                                moves.push(Move {
                                    from: Square::from_index(from_sq_idx as u8),
                                    to: Square::from_index(target_sq_idx as u8),
                                    piece: Piece::Pawn,
                                    promotion: Some(promo_piece),
                                    captured_piece: captured_piece_type,
                                    flags: Flags::PromotionCapture,
                                });
                            }
                        } else {
                            moves.push(Move {
                                from: Square::from_index(from_sq_idx as u8),
                                to: Square::from_index(target_sq_idx as u8),
                                piece: Piece::Pawn,
                                promotion: None,
                                captured_piece: captured_piece_type,
                                flags: Flags::Capture,
                            });
                        }
                    }
                }
            }

            our_pawns &= our_pawns - BitBoard(1); // Clear the least significant bit
        }

        if let Some(ep_sq) = self.en_passant_square {
            let attackers = match self.turn {
                Color::White => {
                    PAWN_ATTACKS[self.turn.opposite() as usize][ep_sq as usize] & self.white_pawns.0
                }
                Color::Black => {
                    PAWN_ATTACKS[self.turn.opposite() as usize][ep_sq as usize] & self.black_pawns.0
                }
            };

            for from_idx in BitBoard(attackers).into_iter() {
                moves.push(Move {
                    from: from_idx,
                    to: ep_sq,
                    piece: Piece::Pawn,
                    promotion: None,
                    captured_piece: Some(Piece::Pawn), // En passant captures a pawn
                    flags: Flags::EnPassant,
                });
            }
        }
    }

    pub fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let (mut knights, our_occupied) = if self.turn == Color::White {
            (self.white_knights.0, self.white_occupied.0)
        } else {
            (self.black_knights.0, self.black_occupied.0)
        };

        while knights != 0 {
            let from_sq_idx = knights.trailing_zeros() as u8;

            for &offset in &KNIGHT_MOVES {
                let target_sq_idx_signed = from_sq_idx as i8 + offset;

                if target_sq_idx_signed >= 0 && target_sq_idx_signed < 64 {
                    let target_sq_idx = target_sq_idx_signed as u8;
                    let target_bit = 1u64 << target_sq_idx;

                    let from_rank = from_sq_idx / 8;
                    let from_file = from_sq_idx % 8;
                    let target_rank = target_sq_idx / 8;
                    let target_file = target_sq_idx % 8;

                    let rank_diff = (from_rank as i8 - target_rank as i8).abs();
                    let file_diff = (from_file as i8 - target_file as i8).abs();

                    if rank_diff == 1 && file_diff == 2 || rank_diff == 2 && file_diff == 1 {
                        if our_occupied & target_bit == 0 {
                            let captured_piece = self
                                .piece_on_square(Square::from_index(target_sq_idx as u8))
                                .map(|(p, _)| p);

                            let mut flags = Flags::Normal;

                            if captured_piece.is_some() {
                                flags = Flags::Capture;
                            }

                            moves.push(Move {
                                from: Square::from_index(from_sq_idx as u8),
                                to: Square::from_index(target_sq_idx as u8),
                                piece: Piece::Knight,
                                promotion: None,
                                captured_piece,
                                flags,
                            })
                        }
                    }
                }
            }

            knights &= knights - 1;
        }
    }

    pub fn generate_king_moves(&self, moves: &mut Vec<Move>) {
        let (mut kings, our_occupied) = if self.turn == Color::White {
            (self.white_king, self.white_occupied)
        } else {
            (self.black_king, self.black_occupied)
        };

        while let Some(from_sq) = Square::try_index(kings.0.trailing_zeros() as usize) {
            let attacks = KING_ATTACKS[from_sq as usize] & !our_occupied;

            kings &= kings - BitBoard(1); // Clear the least significant bit

            for to_sq in attacks.into_iter() {
                if self.is_square_attacked(to_sq, self.turn.opposite()) {
                    continue;
                }

                let cap = self.piece_on_square(to_sq).map(|(p, _)| p);
                let flags = if cap.is_some() {
                    Flags::Capture
                } else {
                    Flags::Normal
                };

                moves.push(Move {
                    from: from_sq,
                    to: to_sq,
                    piece: Piece::King,
                    promotion: None,
                    captured_piece: cap,
                    flags,
                });
            }

            let opp = self.turn.opposite();
            let rights = self.castling_rights;

            match self.turn {
                Color::White if from_sq == Square::E1 => {
                    if rights & W_KINGSIDE_RIGHTS != 0 {
                        if self.empty.has(Square::G1) && self.empty.has(Square::F1) {
                            let opp = self.turn.opposite();
                            if !self.is_square_attacked(Square::E1, opp)
                                && !self.is_square_attacked(Square::F1, opp)
                                && !self.is_square_attacked(Square::G1, opp)
                            {
                                moves.push(Move {
                                    from: Square::E1,
                                    to: Square::G1,
                                    piece: Piece::King,
                                    promotion: None,
                                    captured_piece: None,
                                    flags: Flags::Castling,
                                });
                            }
                        }
                    }

                    if rights & W_QUEENSIDE_RIGHTS != 0 {
                        if self.empty.has(Square::C1)
                            && self.empty.has(Square::D1)
                            && self.empty.has(Square::B1)
                        {
                            let opp = self.turn.opposite();
                            if !self.is_square_attacked(Square::E1, opp)
                                && !self.is_square_attacked(Square::D1, opp)
                                && !self.is_square_attacked(Square::C1, opp)
                            {
                                moves.push(Move {
                                    from: Square::E1,
                                    to: Square::C1,
                                    piece: Piece::King,
                                    promotion: None,
                                    captured_piece: None,
                                    flags: Flags::Castling,
                                });
                            }
                        }
                    }
                }
                Color::Black if from_sq == Square::E8 => {
                    if rights & B_KINGSIDE_RIGHTS != 0 {
                        if self.empty.has(Square::G8) && self.empty.has(Square::F8) {
                            let opp = self.turn.opposite();
                            if !self.is_square_attacked(Square::E8, opp)
                                && !self.is_square_attacked(Square::F8, opp)
                                && !self.is_square_attacked(Square::G8, opp)
                            {
                                moves.push(Move {
                                    from: Square::E8,
                                    to: Square::G8,
                                    piece: Piece::King,
                                    promotion: None,
                                    captured_piece: None,
                                    flags: Flags::Castling,
                                });
                            }
                        }
                    }

                    if rights & B_QUEENSIDE_RIGHTS != 0 {
                        if self.empty.has(Square::C8)
                            && self.empty.has(Square::D8)
                            && self.empty.has(Square::B8)
                        {
                            let opp = self.turn.opposite();
                            if !self.is_square_attacked(Square::E8, opp)
                                && !self.is_square_attacked(Square::D8, opp)
                                && !self.is_square_attacked(Square::C8, opp)
                            {
                                moves.push(Move {
                                    from: Square::E8,
                                    to: Square::C8,
                                    piece: Piece::King,
                                    promotion: None,
                                    captured_piece: None,
                                    flags: Flags::Castling,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn generate_rook_moves(&self, moves: &mut Vec<Move>) {
        let mut rooks = if self.turn == Color::White {
            self.white_rooks
        } else {
            self.black_rooks
        };

        let blockers = self.occupied;

        while rooks != BitBoard::EMPTY {
            let rook_sq = Square::from_index(rooks.0.trailing_zeros() as u8);
            let mut attacks = get_rook_moves(rook_sq, blockers);

            while attacks != BitBoard::EMPTY {
                let target_sq = Square::from_index(attacks.0.trailing_zeros() as u8);
                let piece_data = self.piece_on_square(target_sq);
                let mut flags = Flags::Normal;

                attacks &= attacks - BitBoard(1);

                if piece_data.is_some() {
                    if piece_data.unwrap().1 == self.turn {
                        continue;
                    }
                    flags = Flags::Capture;
                }

                moves.push(Move {
                    from: rook_sq,
                    to: target_sq,
                    piece: Piece::Rook,
                    promotion: None,
                    captured_piece: piece_data.map(|(p, _)| p),
                    flags,
                });
            }

            rooks &= rooks - BitBoard(1); // Clear the least significant bit
        }
    }

    pub fn generate_bishop_moves(&self, moves: &mut Vec<Move>) {
        let mut bishops = if self.turn == Color::White {
            self.white_bishops
        } else {
            self.black_bishops
        };

        let blockers = self.occupied;

        while bishops != BitBoard::EMPTY {
            let bishop_sq = Square::from_index(bishops.0.trailing_zeros() as u8);
            let mut attacks = get_bishop_moves(bishop_sq, blockers);

            while attacks != BitBoard::EMPTY {
                let target_sq = Square::from_index(attacks.0.trailing_zeros() as u8);
                let piece_data = self.piece_on_square(target_sq);
                attacks &= attacks - BitBoard(1); // Clear the least significant bit

                let mut flags = Flags::Normal;

                if piece_data.is_some() {
                    if piece_data.unwrap().1 == self.turn {
                        continue;
                    }
                    flags = Flags::Capture;
                }

                moves.push(Move {
                    from: bishop_sq,
                    to: target_sq,
                    piece: Piece::Bishop,
                    promotion: None,
                    captured_piece: piece_data.map(|(p, _)| p),
                    flags,
                });
            }

            bishops &= bishops - BitBoard(1); // Clear the least significant bit
        }
    }

    pub fn generate_queen_moves(&self, moves: &mut Vec<Move>) {
        let mut queens = if self.turn == Color::White {
            self.white_queens
        } else {
            self.black_queens
        };
        let blockers = self.occupied;

        while queens != BitBoard::EMPTY {
            let queen_sq = Square::from_index(queens.0.trailing_zeros() as u8);
            let mut attacks = get_queen_moves(queen_sq, blockers);

            while attacks != BitBoard::EMPTY {
                let target_sq = Square::from_index(attacks.0.trailing_zeros() as u8);
                let piece_data = self.piece_on_square(target_sq);

                attacks &= attacks - BitBoard(1);

                let mut flags = Flags::Normal;

                if piece_data.is_some() {
                    if piece_data.unwrap().1 == self.turn {
                        continue;
                    }
                    flags = Flags::Capture;
                }

                moves.push(Move {
                    from: queen_sq,
                    to: target_sq,
                    piece: Piece::Queen,
                    promotion: None,
                    captured_piece: piece_data.map(|(p, _)| p),
                    flags,
                });
            }

            queens &= queens - BitBoard(1); // Clear the least significant bit
        }
    }

    pub fn generate_sliding_moves(&self, moves: &mut Vec<Move>, piece: Piece) {
        let mut piece_bitboard = match piece {
            Piece::Rook => {
                if self.turn == Color::White {
                    self.white_rooks
                } else {
                    self.black_rooks
                }
            }
            Piece::Bishop => {
                if self.turn == Color::White {
                    self.white_bishops
                } else {
                    self.black_bishops
                }
            }
            Piece::Queen => {
                if self.turn == Color::White {
                    self.white_queens
                } else {
                    self.black_queens
                }
            }
            _ => unreachable!("Should not be called with a non sliding piece"),
        };
        let our_occupied = if self.turn == Color::White {
            self.white_occupied
        } else {
            self.black_occupied
        };

        let mut piece_bb = piece_bitboard.0;

        while piece_bb != 0 {
            let from_sq_idx = piece_bb.trailing_zeros() as u8;
            let start_dir_idx = if piece == Piece::Bishop { 4 } else { 0 };
            let end_dir_idx = if piece == Piece::Rook { 4 } else { 8 };
            piece_bb &= piece_bb - 1;

            let mut direction_idx = start_dir_idx;
            while direction_idx < end_dir_idx {
                for n in 0..NumSquaresToTheEdge[from_sq_idx as usize][direction_idx] {
                    let target_sq_idx =
                        from_sq_idx as i8 + DIRECTION_OFFSETS[direction_idx] as i8 * (n + 1) as i8;

                    if target_sq_idx < 0 || target_sq_idx >= 64 {
                        break; // Out of bounds
                    }

                    let target_sq = Square::from_index(target_sq_idx as u8);
                    let target_bit = target_sq.bb().0;

                    if let Some((_, color)) =
                        self.piece_on_square(Square::from_index(target_sq as u8))
                    {
                        if color == self.turn {
                            break;
                        }
                    }

                    if our_occupied.0 & target_bit == 0 {
                        let captured_piece = self.piece_on_square(target_sq).map(|(p, _)| p);
                        let mut flags = Flags::Normal;

                        if captured_piece.is_some() {
                            flags = Flags::Capture;
                        }

                        moves.push(Move {
                            from: Square::from_index(from_sq_idx as u8),
                            to: target_sq,
                            piece: piece,
                            promotion: None,
                            captured_piece,
                            flags,
                        });

                        if captured_piece.is_some() {
                            break;
                        } // Stop if we hit a piece
                    }
                }

                direction_idx += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        consts::Square::{D5, E4, E5, F6},
        moves,
    };

    use super::*;

    #[test]
    fn test_generate_pawn_moves() {
        let board = Board::default();
        let mut moves: Vec<Move> = Vec::new();

        board.generate_pawn_moves(&mut moves);
        assert_eq!(moves.len(), 16);
    }

    #[test]
    fn test_alot_of_pawn_moves() {
        let board1 = Board::from_fen(
            "rnbqk2r/5pb1/5n2/ppp1p1pp/PPPp2PP/1Q1P1N2/4PPB1/RNB1K2R w KQkq - 1 11",
        )
        .unwrap();

        let moves1: Vec<Move> = board1
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::Pawn)
            .collect();

        assert_eq!(moves1.len(), 8);

        let board2 = Board::from_fen(
            "rnbqk2r/5pb1/5n2/ppp1p1pp/PPPpP1PP/1Q1P1N2/5PB1/RNB1K2R b KQkq e3 0 11",
        )
        .unwrap();

        let moves2: Vec<Move> = board2
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::Pawn)
            .collect();

        let mut is_there_en_passant = false;
        for mv in moves2 {
            if mv.flags == Flags::EnPassant {
                is_there_en_passant = true;
            }
        }

        assert_eq!(is_there_en_passant, true);
    }

    #[test]
    fn test_pawn_capture() {
        let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
            .unwrap();

        let mut moves: Vec<Move> = Vec::new();
        board.generate_pawn_moves(&mut moves);

        let piece = board.piece_on_square(Square::E4).unwrap();

        assert!(piece.0 == Piece::Pawn);
        assert!(piece.1 == Color::White);

        for mv in moves {
            if mv.from == Square::E4 {
                assert!(mv.to == Square::E5 || mv.to == Square::D5);
            }
        }
    }

    #[test]
    fn test_en_passant_capture() {
        let board =
            Board::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
                .unwrap();

        let mut moves: Vec<Move> = Vec::new();
        board.generate_pawn_moves(&mut moves);

        let piece = board.piece_on_square(Square::E5).map(|(p, _)| p).unwrap();

        assert!(piece == Piece::Pawn);

        for mv in moves {
            if mv.flags == Flags::EnPassant {
                assert_eq!(mv.from, Square::E5);
                assert_eq!(mv.to, Square::F6);
            }
        }

        let board1 =
            Board::from_fen("r1bqkbnr/ppppp1pp/2n5/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
                .unwrap();

        let mut moves1: Vec<Move> = board1
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::Pawn && mv.from == Square::E5)
            .collect();

        println!("Generated moves: {:?}", moves1);
        assert_eq!(moves1.len(), 2);
    }

    #[test]
    fn test_initial_knight_moves() {
        let board = Board::default();
        let mut moves: Vec<moves::Move> = Vec::new();
        board.generate_knight_moves(&mut moves);
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_centralized_knight_moves() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/3N4/8/PPPPPPPP/RNBQKB1R w KQkq - 4 3").unwrap();
        let mut moves: Vec<moves::Move> = Vec::new();
        board.generate_knight_moves(&mut moves);

        assert_eq!(moves.len(), 8);

        let board2 =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2")
                .unwrap();
        let mut moves2: Vec<moves::Move> = board2
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::Knight)
            .collect();

        assert_eq!(moves2.len(), 7);
    }

    #[test]
    fn test_bishop_moves() {
        let board =
            Board::from_fen("rnbqkbnr/ppppp2p/5p2/6p1/2B5/4P3/PPPP1PPP/RNBQK1NR w KQkq - 0 3")
                .unwrap();

        let mut moves: Vec<moves::Move> = Vec::new();

        board.generate_sliding_moves(&mut moves, Piece::Bishop);

        assert_eq!(moves.len(), 10);
    }

    #[test]
    fn test_queen_moves() {
        let board =
            Board::from_fen("rnbqkbnr/ppp1p2p/3p1p2/6p1/2B3Q1/4P3/PPPP1PPP/RNB1K1NR w KQkq - 0 4")
                .unwrap();

        let mut moves: Vec<moves::Move> = Vec::new();

        board.generate_sliding_moves(&mut moves, Piece::Queen);

        assert_eq!(moves.len(), 15);

        let board1 =
            Board::from_fen("rnbq1b1r/1pp1kppp/p4n2/3Qp3/3P4/5N2/PPP1PPPP/RNB1KB1R w KQ - 0 6")
                .unwrap();

        let mut moves1 = board1
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::Queen)
            .collect::<Vec<Move>>();

        println!("Generated moves: {:?}", moves1);

        assert_eq!(moves1.len(), 14);
    }

    #[test]
    fn test_rook_moves() {
        let board =
            Board::from_fen("rnbqkbn1/ppp4r/3p1p2/4p1pp/2B5/3PP2Q/PPP1NPPP/RNB2RK1 b q - 0 7")
                .unwrap();

        let mut moves: Vec<moves::Move> = Vec::new();

        board.generate_rook_moves(&mut moves);

        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn test_king_moves() {
        let board =
            Board::from_fen("rnbq1bn1/ppp1k2r/3p1p2/4p1pp/2B5/3PPQ2/PPP1NPPP/RNB2RK1 b - - 2 8")
                .unwrap();

        let mut moves: Vec<moves::Move> = Vec::new();
        board.generate_king_moves(&mut moves);
        assert_eq!(moves.len(), 2);
    }

    #[test]
    fn test_king_in_front_of_king_moves() {
        let board =
            Board::from_fen("rnbq1bnr/pppp1ppp/2k5/4p3/2K1P3/8/PPPP1PPP/RNBQ1BNR w - - 6 5")
                .unwrap();

        let mut moves: Vec<moves::Move> = Vec::new();
        board.generate_king_moves(&mut moves);

        assert_eq!(moves.len(), 3);
    }

    #[test]
    fn test_castling_moves() {
        let mut board1 = Board::from_fen(
            "r1bqkb1r/2pp1ppp/p1n2n2/1p2p3/4P3/1B3N2/PPPP1PPP/RNBQK2R w KQkq - 2 6",
        )
        .unwrap();
        let moves1: Vec<moves::Move> = board1
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::King)
            .collect();

        let castling_move = moves1.iter().find(|mv| mv.flags == Flags::Castling);
        if castling_move.is_some() {
            let mv = castling_move.unwrap();

            board1.make_move(&mv);
        }

        let mut board2 =
            Board::from_fen("r1bqk2r/ppp2ppp/2nb1n2/3pp3/3PP3/2NQB3/PPP2PPP/R3KBNR w KQkq - 6 6")
                .unwrap();
        let moves2: Vec<moves::Move> = board2
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.piece == Piece::King)
            .collect();

        let castling_move = moves2.iter().find(|mv| mv.flags == Flags::Castling);

        assert!(castling_move.is_some(), "Castling move should be available");
    }
}
