use crate::{
    bitboard::BitBoard,
    board::{self, Board, Color},
    consts::{
        File, Rank, Square, BISHOP_ATTACKS, DIRECTION_OFFSETS, KING_ATTACKS, KING_MOVES, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS
    },
    sliding_pieces::{get_bishop_moves, get_rook_moves},
};

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, &'static str> {
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err("FEN must have 6 parts");
        }

        let piece_placement = parts[0];
        let active_color = parts[1];
        let castling_rights_str = parts[2];
        let en_passant_sq_str = parts[3];
        let halfmove_clock_str = parts[4];
        let fullmove_number_str = parts[5];

        let mut board = Board {
            white_pawns: BitBoard(0),
            white_knights: BitBoard(0),
            white_bishops: BitBoard(0),
            white_rooks: BitBoard(0),
            white_queens: BitBoard(0),
            white_king: BitBoard(0),
            black_pawns: BitBoard(0),
            black_knights: BitBoard(0),
            black_bishops: BitBoard(0),
            black_rooks: BitBoard(0),
            black_queens: BitBoard(0),
            black_king: BitBoard(0),
            white_occupied: BitBoard(0),
            black_occupied: BitBoard(0),
            occupied: BitBoard(0),
            empty: BitBoard(0),
            turn: board::Color::White,
            castling_rights: 0b1111,
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0,
            history: Vec::new(),
        };

        let mut rank = 7;
        let mut file = 0;
        for c in piece_placement.chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if c.is_digit(10) {
                file += c.to_digit(10).unwrap() as u8;
            } else {
                let square_idx = rank * 8 + file;
                let bit = 1u64 << square_idx;
                match c {
                    'P' => board.white_pawns.0 |= bit,
                    'N' => board.white_knights.0 |= bit,
                    'B' => board.white_bishops.0 |= bit,
                    'R' => board.white_rooks.0 |= bit,
                    'Q' => board.white_queens.0 |= bit,
                    'K' => board.white_king.0 |= bit,
                    'p' => board.black_pawns.0 |= bit,
                    'n' => board.black_knights.0 |= bit,
                    'b' => board.black_bishops.0 |= bit,
                    'r' => board.black_rooks.0 |= bit,
                    'q' => board.black_queens.0 |= bit,
                    'k' => board.black_king.0 |= bit,
                    _ => return Err("Invalid FEN string: Invalid piece character"),
                }
                file += 1;
            }
        }

        board.white_occupied = board.white_pawns
            | board.white_knights
            | board.white_bishops
            | board.white_rooks
            | board.white_queens
            | board.white_king;
        board.black_occupied = board.black_pawns
            | board.black_knights
            | board.black_bishops
            | board.black_rooks
            | board.black_queens
            | board.black_king;

        board.occupied = board.white_occupied | board.black_occupied;
        board.empty = !board.occupied;

        if active_color == "w" {
            board.turn = Color::White;
        } else if active_color == "b" {
            board.turn = Color::Black;
        } else {
            return Err("Invalid FEN string: Invalid active color");
        }

        for c in castling_rights_str.chars() {
            match c {
                'K' => board.castling_rights |= 0b0001, // White kingside
                'Q' => board.castling_rights |= 0b0010, // White queenside
                'k' => board.castling_rights |= 0b0100, // Black kingside
                'q' => board.castling_rights |= 0b1000, // Black queenside
                '-' => continue,
                _ => return Err("Invalid FEN string: Invalid castling rights"),
            }
        }

        if en_passant_sq_str != "-" {
            if en_passant_sq_str.len() != 2 {
                return Err("Invalid FEN string: Invalid en passant square");
            }
            let file_char = en_passant_sq_str.chars().next().unwrap();
            let rank_char = en_passant_sq_str.chars().nth(1).unwrap();

            let file_idx = match file_char {
                'a' => File::A,
                'b' => File::B,
                'c' => File::C,
                'd' => File::D,
                'e' => File::E,
                'f' => File::F,
                'g' => File::G,
                'h' => File::H,
                _ => return Err("Invalid FEN string: Invalid en passant square"),
            };

            let rank_idx = match rank_char.to_digit(10) {
                Some(1) => Rank::First,
                Some(2) => Rank::Second,
                Some(3) => Rank::Third,
                Some(4) => Rank::Fourth,
                Some(5) => Rank::Fifth,
                Some(6) => Rank::Sixth,
                Some(7) => Rank::Seventh,
                Some(8) => Rank::Eighth,
                _ => return Err("Invalid FEN string: Invalid en passant square"),
            };

            board.en_passant_square = Some(Square::new(file_idx, rank_idx).unwrap());
        }

        board.halfmove_clock = halfmove_clock_str
            .parse::<u8>()
            .map_err(|_| "Invalid FEN string: Invalid halfmove clock")?;
        board.fullmove_number = fullmove_number_str
            .parse::<u16>()
            .map_err(|_| "Invalid FEN string: Invalid fullmove number")?;

        Ok(board)
    }

    pub fn is_square_attacked(&self, sq: Square, attacking_color: Color) -> bool {
        let (pawns, knights, bishops, rooks, queens, king) = match attacking_color {
            Color::White => (
                self.white_pawns,
                self.white_knights,
                self.white_bishops,
                self.white_rooks,
                self.white_queens,
                self.white_king,
            ),
            Color::Black => (
                self.black_pawns,
                self.black_knights,
                self.black_bishops,
                self.black_rooks,
                self.black_queens,
                self.black_king,
            ),
        };



        if PAWN_ATTACKS[attacking_color.opposite() as usize][sq as usize] & pawns.0 != 0 {
            return true;
        }

        if KNIGHT_ATTACKS[sq as usize] & knights.0 != 0 {
            return true;
        }

        if KING_ATTACKS[sq as usize] & king != BitBoard::EMPTY {
            return true;
        }

        let blockers = self.occupied;

        if get_bishop_moves(sq, blockers) & (bishops | queens) != BitBoard::EMPTY {
            return true;
        }

        if get_rook_moves(sq, blockers) & (rooks | queens) != BitBoard::EMPTY {
            return true;
        }

        false
    }

    pub fn is_king_in_check(&self, attacking_color: Color) -> bool {
        let attacked_king = match attacking_color {
            Color::White => self.black_king,
            Color::Black => self.white_king,
        };

        if attacked_king.0 == 0 {
            return false;
        }

        let sq = Square::from_index(attacked_king.0.trailing_zeros() as u8);

        self.is_square_attacked(sq, attacking_color)
    }

    // fn get_rook_attacks(&self, sq: Square, blockers: u64) -> u64 {
    //     let sq_idx = sq as usize;
    //     let mut attacks = 0u64;

    //     let north_ray =
    // }

    pub fn is_insufficient_material(&self) -> bool {
        let mut white_minor = 0;
        let mut black_minor = 0;
        let mut white_major = 0;
        let mut black_major = 0;

        if self.white_pawns != BitBoard::EMPTY || self.black_pawns != BitBoard::EMPTY {
            return false; // Presence of pawns means insufficient material is not possible
        }

        white_major += self.white_rooks.count() + self.white_queens.count();
        black_major += self.black_rooks.count() + self.black_queens.count();

        if white_major > 0 && black_major > 0 {
            return false;
        }

        white_minor += self.white_knights.count() + self.white_bishops.count();
        black_minor += self.black_knights.count() + self.black_bishops.count();

        if white_minor == 0 && black_minor == 0 { return true; }

        if white_minor == 1 && black_minor == 0 && self.white_queens.is_empty() && self.white_rooks.is_empty() { return true; }
        if black_minor == 1 && white_minor == 0 && self.black_queens.is_empty() && self.black_rooks.is_empty() { return true; }

        if white_minor == 1 && self.white_bishops.count() == 1 && black_minor == 0 { return true; }
        if black_minor == 1 && self.black_bishops.count() == 1 && white_minor == 0 { return true; }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::moves::Move;

    use super::*;

    #[test]
    fn test_from_fen() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        assert_eq!(board.white_pawns.0, 0x000000000000FF00);
        assert_eq!(board.white_knights.0, 0x0000000000000042);
        assert_eq!(board.white_bishops.0, 0x0000000000000024);
        assert_eq!(board.white_rooks.0, 0x0000000000000081);
        assert_eq!(board.white_queens.0, 0x0000000000000008);
        assert_eq!(board.white_king.0, 0x0000000000000010);

        assert_eq!(board.black_pawns.0, 0x00FF000000000000);
        assert_eq!(board.black_knights.0, 0x4200000000000000);
        assert_eq!(board.black_bishops.0, 0x2400000000000000);
        assert_eq!(board.black_rooks.0, 0x8100000000000000);
        assert_eq!(board.black_queens.0, 0x0800000000000000);
        assert_eq!(board.black_king.0, 0x1000000000000000);

        assert_eq!(board.white_occupied.0, 0x000000000000FFFF);
        assert_eq!(board.black_occupied.0, 0xFFFF000000000000);
        assert_eq!(board.occupied.0, 0xFFFF00000000FFFF);
        assert_eq!(board.empty.0, !0xFFFF00000000FFFF);

        assert_eq!(board.turn, Color::White);
        assert_eq!(board.castling_rights, 15);
        assert_eq!(board.en_passant_square, None);
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 1);
        assert_eq!(board.zobrist_hash, 0);
    }

    #[test]
    fn test_is_white_king_in_check() {
        let board =
            Board::from_fen("rnb1kb1r/ppp2ppp/1q3n2/3pp3/4P3/1K6/PPPP1PPP/RNBQ1BNR w kq - 4 6")
                .unwrap();

        assert!(board.is_king_in_check(Color::Black));
    }

    #[test]
    fn test_is_black_king_in_check() {
        let board =
            Board::from_fen("rnb1kb1r/ppp2ppp/1q3n2/1B1pp3/4P3/1K6/PPPP1PPP/RNBQ2NR b kq - 5 6")
                .unwrap();

        assert_eq!(board.is_king_in_check(Color::White), true);
    }

    #[test]
    fn test_is_king_in_check_even_tho_its_pined() {
        let board =
            Board::from_fen("rnb1kb1r/pp3ppp/1qp2n2/1B1pp3/4P3/1K5P/PPPP1PP1/RNBQ2NR b kq - 0 7")
                .unwrap();

        assert_eq!(board.is_king_in_check(Color::White), false);
    }

    #[test]
    fn test_king_moves_knight_near() {
        let board = Board::from_fen("r1bq1bnr/ppp2ppp/1nkp4/4p3/1K2P3/6PP/PPPP1P2/RNBQ1BNR w - - 3 8").unwrap();

        let mut moves: Vec<Move> = Vec::new();

        board.generate_king_moves(&mut moves);

        println!("Generated moves: {:?}", moves);
        assert_eq!(moves.len(), 4);
    }
}
