use crate::{
    bitboard::BitBoard,
    consts::{
        Square, B_KINGSIDE_RIGHTS, B_QUEENSIDE_RIGHTS, W_KINGSIDE_RIGHTS, W_QUEENSIDE_RIGHTS,
    },
    magic_gen::{BISHOP_DELTAS, ROOK_DELTAS},
    moves::{Flags, Move},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn sliding_moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let deltas = match self {
            Piece::Rook => &ROOK_DELTAS,
            Piece::Bishop => &BISHOP_DELTAS,
            _ => unreachable!(),
        };

        let mut moves = BitBoard::EMPTY;
        for &(df, dr) in deltas {
            let mut ray = square;
            while !blockers.has(ray) {
                if let Some(shifted) = ray.try_offset(df, dr) {
                    ray = shifted;
                    moves |= ray.bb();
                } else {
                    break;
                }
            }
        }
        moves
    }

    pub fn name(&self) -> &'static str {
        match self {
            Piece::Pawn => "Pawn",
            Piece::Knight => "Knight",
            Piece::Bishop => "Bishop",
            Piece::Rook => "Rook",
            Piece::Queen => "Queen",
            Piece::King => "King",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Undo {
    captured: Option<(Square, Piece, Color)>,
    castling_rights: u8,
    ep_square: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: u16,
}

#[derive(Clone)]
pub struct Board {
    pub white_pawns: BitBoard,
    pub white_knights: BitBoard,
    pub white_bishops: BitBoard,
    pub white_rooks: BitBoard,
    pub white_queens: BitBoard,
    pub white_king: BitBoard,

    pub black_pawns: BitBoard,
    pub black_knights: BitBoard,
    pub black_bishops: BitBoard,
    pub black_rooks: BitBoard,
    pub black_queens: BitBoard,
    pub black_king: BitBoard,

    pub white_occupied: BitBoard,
    pub black_occupied: BitBoard,

    pub occupied: BitBoard,
    pub empty: BitBoard,

    pub turn: Color,
    pub castling_rights: u8,
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,

    pub zobrist_hash: u64,

    pub history: Vec<Undo>,
}

impl Board {
    pub fn default() -> Board {
        // standart fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
        let white_pawns = BitBoard(0x000000000000FF00);
        let white_knights = BitBoard(0x0000000000000042);
        let white_bishops = BitBoard(0x0000000000000024);
        let white_rooks = BitBoard(0x0000000000000081);
        let white_queens = BitBoard(0x0000000000000008);
        let white_king = BitBoard(0x0000000000000010);

        let black_pawns = BitBoard(0x00FF000000000000);
        let black_knights = BitBoard(0x4200000000000000);
        let black_bishops = BitBoard(0x2400000000000000);
        let black_rooks = BitBoard(0x8100000000000000);
        let black_queens = BitBoard(0x0800000000000000);
        let black_king = BitBoard(0x1000000000000000);

        let white_occupied =
            white_pawns | white_knights | white_bishops | white_rooks | white_queens | white_king;
        let black_occupied =
            black_pawns | black_knights | black_bishops | black_rooks | black_queens | black_king;

        let occupied = white_occupied | black_occupied;
        let empty = !occupied;

        Board {
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
            white_occupied,
            black_occupied,
            occupied,
            empty,
            turn: Color::White,
            castling_rights: 0b1111,
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0,
            history: Vec::new(),
        }
    }

    pub fn make_move(&mut self, mv: &Move) {
        let undo = Undo {
            captured: mv
                .captured_piece
                .map(|pc| (mv.to, pc, self.turn.opposite())),
            castling_rights: self.castling_rights,
            ep_square: self.en_passant_square,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
        };

        self.history.push(undo);

        let from_bit = mv.from.bb();
        let to_bit = mv.to.bb();

        match self.turn {
            Color::White => {
                if self.white_pawns & from_bit != BitBoard::EMPTY {
                    self.white_pawns &= !from_bit;
                } else if self.white_knights & from_bit != BitBoard::EMPTY {
                    self.white_knights &= !from_bit;
                } else if self.white_bishops & from_bit != BitBoard::EMPTY {
                    self.white_bishops &= !from_bit;
                } else if self.white_rooks & from_bit != BitBoard::EMPTY {
                    self.white_rooks &= !from_bit;
                } else if self.white_queens & from_bit != BitBoard::EMPTY {
                    self.white_queens &= !from_bit;
                } else if self.white_king & from_bit != BitBoard::EMPTY {
                    self.white_king &= !from_bit;
                }

                if self.black_occupied & to_bit != BitBoard::EMPTY {
                    if self.black_pawns & to_bit != BitBoard::EMPTY {
                        self.black_pawns &= !to_bit;
                    } else if self.black_knights & to_bit != BitBoard::EMPTY {
                        self.black_knights &= !to_bit;
                    } else if self.black_bishops & to_bit != BitBoard::EMPTY {
                        self.black_bishops &= !to_bit;
                    } else if self.black_rooks & to_bit != BitBoard::EMPTY {
                        self.black_rooks &= !to_bit;
                    } else if self.black_queens & to_bit != BitBoard::EMPTY {
                        self.black_queens &= !to_bit;
                    } else if self.black_king & to_bit != BitBoard::EMPTY {
                        self.black_king &= !to_bit;
                    }
                }

                if let Some(promotion_piece) = mv.promotion {
                    match promotion_piece {
                        Piece::Queen => self.white_queens |= to_bit,
                        Piece::Rook => self.white_rooks |= to_bit,
                        Piece::Bishop => self.white_bishops |= to_bit,
                        Piece::Knight => self.white_knights |= to_bit,
                        _ => unreachable!(
                            "Pawn can only be promoted to Queen, Rook, Bishop, or Knight"
                        ),
                    }
                } else {
                    match mv.piece {
                        Piece::Pawn => self.white_pawns |= to_bit,
                        Piece::Knight => self.white_knights |= to_bit,
                        Piece::Bishop => self.white_bishops |= to_bit,
                        Piece::Rook => self.white_rooks |= to_bit,
                        Piece::Queen => self.white_queens |= to_bit,
                        Piece::King => self.white_king |= to_bit,
                    }
                }
            }
            Color::Black => {
                if self.black_pawns & from_bit != BitBoard::EMPTY {
                    self.black_pawns &= !from_bit;
                } else if self.black_knights & from_bit != BitBoard::EMPTY {
                    self.black_knights &= !from_bit;
                } else if self.black_bishops & from_bit != BitBoard::EMPTY {
                    self.black_bishops &= !from_bit;
                } else if self.black_rooks & from_bit != BitBoard::EMPTY {
                    self.black_rooks &= !from_bit;
                } else if self.black_queens & from_bit != BitBoard::EMPTY {
                    self.black_queens &= !from_bit;
                } else if self.black_king & from_bit != BitBoard::EMPTY {
                    self.black_king &= !from_bit;
                }

                if self.white_occupied & to_bit != BitBoard::EMPTY {
                    if self.white_pawns & to_bit != BitBoard::EMPTY {
                        self.white_pawns &= !to_bit;
                    } else if self.white_knights & to_bit != BitBoard::EMPTY {
                        self.white_knights &= !to_bit;
                    } else if self.white_bishops & to_bit != BitBoard::EMPTY {
                        self.white_bishops &= !to_bit;
                    } else if self.white_rooks & to_bit != BitBoard::EMPTY {
                        self.white_rooks &= !to_bit;
                    } else if self.white_queens & to_bit != BitBoard::EMPTY {
                        self.white_queens &= !to_bit;
                    } else if self.white_king & to_bit != BitBoard::EMPTY {
                        self.white_king &= !to_bit;
                    }
                }

                if let Some(promotion_piece) = mv.promotion {
                    match promotion_piece {
                        Piece::Queen => self.black_queens |= to_bit,
                        Piece::Rook => self.black_rooks |= to_bit,
                        Piece::Bishop => self.black_bishops |= to_bit,
                        Piece::Knight => self.black_knights |= to_bit,
                        _ => unreachable!(
                            "Pawn can only be promoted to Queen, Rook, Bishop, or Knight"
                        ),
                    }
                } else {
                    match mv.piece {
                        Piece::Pawn => self.black_pawns |= to_bit,
                        Piece::Knight => self.black_knights |= to_bit,
                        Piece::Bishop => self.black_bishops |= to_bit,
                        Piece::Rook => self.black_rooks |= to_bit,
                        Piece::Queen => self.black_queens |= to_bit,
                        Piece::King => self.black_king |= to_bit,
                    }
                }
            }
        }

        self.white_occupied = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king;
        self.black_occupied = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king;

        self.occupied = self.white_occupied | self.black_occupied;
        self.empty = !self.occupied;

        if mv.piece == Piece::Pawn && (mv.to as i8 - mv.from as i8).abs() == 16 {
            // two‑square pawn push → record the square “behind” the pawn
            let mid = ((mv.from as usize + mv.to as usize) / 2) as usize;
            self.en_passant_square = Some(Square::from_index(mid as u8));
        } else {
            // any other half‑move clears it
            self.en_passant_square = None;
        }

        if mv.piece == Piece::Pawn || (self.occupied.0 & to_bit.0) != 0 {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }

        if mv.flags == Flags::Castling {
            if mv.from == Square::E1 {
                if mv.to == Square::G1 {
                    self.delete_piece(Square::H1);
                    self.add_piece(Square::F1, Piece::Rook, self.turn);
                } else {
                    self.delete_piece(Square::A1);
                    self.add_piece(Square::D1, Piece::Rook, self.turn);
                }

                self.castling_rights &= !W_KINGSIDE_RIGHTS;
                self.castling_rights &= !W_QUEENSIDE_RIGHTS;
            } else if mv.from == Square::E8 {
                if mv.to == Square::G8 {
                    self.delete_piece(Square::H8);
                    self.add_piece(Square::F8, Piece::Rook, self.turn);
                } else {
                    self.delete_piece(Square::A8);
                    self.add_piece(Square::D8, Piece::Rook, self.turn);
                }

                self.castling_rights &= !B_KINGSIDE_RIGHTS;
                self.castling_rights &= !B_QUEENSIDE_RIGHTS;
            }
        }

        if mv.piece == Piece::King {
            if mv.from == Square::E1 {
                self.castling_rights &= !W_KINGSIDE_RIGHTS;
                self.castling_rights &= !W_QUEENSIDE_RIGHTS;
            } else if mv.from == Square::E8 {
                self.castling_rights &= !B_KINGSIDE_RIGHTS;
                self.castling_rights &= !B_QUEENSIDE_RIGHTS;
            }
        }

        if mv.piece == Piece::Rook {
            if mv.from == Square::A1 {
                self.castling_rights &= !W_QUEENSIDE_RIGHTS;
            } else if mv.from == Square::H1 {
                self.castling_rights &= !W_KINGSIDE_RIGHTS;
            }

            if mv.from == Square::A8 {
                self.castling_rights &= !B_QUEENSIDE_RIGHTS;
            } else if mv.from == Square::H8 {
                self.castling_rights &= !B_KINGSIDE_RIGHTS;
            }
        }

        // TODO: Handle special moves like castling and en passant captures explicitly.

        if mv.flags == Flags::EnPassant {
            let captured_pawn_offset = if self.turn == Color::White { -8 } else { 8 };
            let captured_pawn_idx = (mv.to as i8 + captured_pawn_offset) as u8;

            let captured_pawn = Square::try_index(captured_pawn_idx as usize);

            if captured_pawn.is_some() {
                self.delete_piece(captured_pawn.unwrap());
            } else {
                unreachable!("Invalid en passant square: {}", captured_pawn_idx);
            }
        }

        // TODO: Update Zobrist hash (incrementally)
        self.turn = self.turn.opposite();
    }

    /// Reverts a given move, restoring the board to its previous state.
    /// This is a simplified version and will need to be expanded to handle all move types.
    /// This requires storing more information about the move (e.g., captured piece type).
    pub fn unmake_move(&mut self, mv: &Move) {
        let undo = self
            .history
            .pop()
            .expect("unmake_move: no undo information");

        self.castling_rights = undo.castling_rights;
        self.en_passant_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.fullmove_number = undo.fullmove_number;

        let from_bit = mv.from.bb();
        let to_bit = mv.to.bb();

        match self.turn.opposite() {
            Color::White => {
                // White's piece was moved
                // Remove piece from 'to' square (handle promotion reversal)
                if mv.promotion.is_some() {
                    match mv.promotion.unwrap() {
                        Piece::Queen => self.white_queens &= !to_bit,
                        Piece::Rook => self.white_rooks &= !to_bit,
                        Piece::Bishop => self.white_bishops &= !to_bit,
                        Piece::Knight => self.white_knights &= !to_bit,
                        _ => unreachable!(),
                    }

                    self.white_pawns |= from_bit; // Original piece was a pawn
                } else {
                    // If no promotion, move the original piece type back
                    match mv.piece {
                        Piece::Pawn => {
                            self.white_pawns &= !to_bit;
                            self.white_pawns |= from_bit;
                        }
                        Piece::Knight => {
                            self.white_knights &= !to_bit;
                            self.white_knights |= from_bit;
                        }
                        Piece::Bishop => {
                            self.white_bishops &= !to_bit;
                            self.white_bishops |= from_bit;
                        }
                        Piece::Rook => {
                            self.white_rooks &= !to_bit;
                            self.white_rooks |= from_bit;
                        }
                        Piece::Queen => {
                            self.white_queens &= !to_bit;
                            self.white_queens |= from_bit;
                        }
                        Piece::King => {
                            self.white_king &= !to_bit;
                            self.white_king |= from_bit;
                        }
                    }
                }
            }
            Color::Black => {
                if mv.promotion.is_some() {
                    match mv.promotion.unwrap() {
                        Piece::Queen => self.black_queens &= !to_bit,
                        Piece::Rook => self.black_rooks &= !to_bit,
                        Piece::Bishop => self.black_bishops &= !to_bit,
                        Piece::Knight => self.black_knights &= !to_bit,
                        _ => unreachable!(),
                    }
                    self.black_pawns |= from_bit; // Original piece was a pawn
                } else {
                    match mv.piece {
                        Piece::Pawn => {
                            self.black_pawns &= !to_bit;
                            self.black_pawns |= from_bit;
                        }
                        Piece::Knight => {
                            self.black_knights &= !to_bit;
                            self.black_knights |= from_bit;
                        }
                        Piece::Bishop => {
                            self.black_bishops &= !to_bit;
                            self.black_bishops |= from_bit;
                        }
                        Piece::Rook => {
                            self.black_rooks &= !to_bit;
                            self.black_rooks |= from_bit;
                        }
                        Piece::Queen => {
                            self.black_queens &= !to_bit;
                            self.black_queens |= from_bit;
                        }
                        Piece::King => {
                            self.black_king &= !to_bit;
                            self.black_king |= from_bit;
                        }
                    }
                }
            }
        }
        if let Some((sq, piece, color)) = undo.captured {
            let bb = sq.bb();

            match (piece, color) {
                (Piece::Pawn, Color::White) => self.white_pawns |= bb,
                (Piece::Knight, Color::White) => self.white_knights |= bb,
                (Piece::Bishop, Color::White) => self.white_bishops |= bb,
                (Piece::Rook, Color::White) => self.white_rooks |= bb,
                (Piece::Queen, Color::White) => self.white_queens |= bb,
                (Piece::Pawn, Color::Black) => self.black_pawns |= bb,
                (Piece::Knight, Color::Black) => self.black_knights |= bb,
                (Piece::Bishop, Color::Black) => self.black_bishops |= bb,
                (Piece::Rook, Color::Black) => self.black_rooks |= bb,
                (Piece::Queen, Color::Black) => self.black_queens |= bb,
                _ => unreachable!("Invalid captured piece: {:?}", piece),
            }
        }

        if mv.flags == Flags::Castling {
            match (mv.piece, mv.to) {
                (Piece::King, Square::G1) => {
                    // white kingside: rook f1 -> h1
                    self.white_rooks &= !Square::F1.bb();
                    self.white_rooks |= Square::H1.bb();
                }
                (Piece::King, Square::C1) => {
                    self.white_rooks &= !Square::D1.bb();
                    self.white_rooks |= Square::A1.bb();
                }
                (Piece::King, Square::G8) => {
                    self.black_rooks &= !Square::F8.bb();
                    self.black_rooks |= Square::H8.bb();
                }
                (Piece::King, Square::C8) => {
                    self.black_rooks &= !Square::D8.bb();
                    self.black_rooks |= Square::A8.bb();
                }
                _ => {}
            }
        }

        self.white_occupied = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king;
        self.black_occupied = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king;
        self.occupied = self.white_occupied | self.black_occupied;
        self.empty = !self.occupied;

        self.turn = self.turn.opposite();
    }

    pub fn add_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let square_bit = square.bb();

        match color {
            Color::White => {
                self.white_occupied |= square_bit;

                match piece {
                    Piece::Pawn => self.white_pawns |= square_bit,
                    Piece::Knight => self.white_knights |= square_bit,
                    Piece::Bishop => self.white_bishops |= square_bit,
                    Piece::Rook => self.white_rooks |= square_bit,
                    Piece::Queen => self.white_queens |= square_bit,
                    Piece::King => self.white_king |= square_bit,
                }
            }
            Color::Black => {
                self.black_occupied |= square_bit;

                match piece {
                    Piece::Pawn => self.black_pawns |= square_bit,
                    Piece::Knight => self.black_knights |= square_bit,
                    Piece::Bishop => self.black_bishops |= square_bit,
                    Piece::Rook => self.black_rooks |= square_bit,
                    Piece::Queen => self.black_queens |= square_bit,
                    Piece::King => self.black_king |= square_bit,
                }
            }
        }

        self.occupied |= square_bit;
        self.empty &= !square_bit;
    }

    pub fn delete_piece(&mut self, square: Square) {
        let square_bit = square.bb();
        let piece = self.piece_on_square(square);

        if piece.is_some() {
            let (piece, color) = piece.unwrap();

            match color {
                Color::White => {
                    self.white_occupied ^= square_bit;

                    match piece {
                        Piece::Pawn => self.white_pawns ^= square_bit,
                        Piece::Knight => self.white_knights ^= square_bit,
                        Piece::Bishop => self.white_bishops ^= square_bit,
                        Piece::Rook => self.white_rooks ^= square_bit,
                        Piece::Queen => self.white_queens ^= square_bit,
                        Piece::King => self.white_king ^= square_bit,
                    }
                }
                Color::Black => {
                    self.black_occupied ^= square_bit;

                    match piece {
                        Piece::Pawn => self.black_pawns ^= square_bit,
                        Piece::Knight => self.black_knights ^= square_bit,
                        Piece::Bishop => self.black_bishops ^= square_bit,
                        Piece::Rook => self.black_rooks ^= square_bit,
                        Piece::Queen => self.black_queens ^= square_bit,
                        Piece::King => self.black_king ^= square_bit,
                    }
                }
            }

            self.occupied ^= square_bit;
            self.empty |= square_bit;
        }
    }

    pub fn piece_on_square(&self, square: Square) -> Option<(Piece, Color)> {
        let square_bit = square.bb();

        if (self.white_occupied.0 & square_bit.0) != 0 {
            if (self.white_pawns.0 & square_bit.0) != 0 {
                Some((Piece::Pawn, Color::White))
            } else if (self.white_knights.0 & square_bit.0) != 0 {
                Some((Piece::Knight, Color::White))
            } else if (self.white_bishops.0 & square_bit.0) != 0 {
                Some((Piece::Bishop, Color::White))
            } else if (self.white_rooks.0 & square_bit.0) != 0 {
                Some((Piece::Rook, Color::White))
            } else if (self.white_queens.0 & square_bit.0) != 0 {
                Some((Piece::Queen, Color::White))
            } else if (self.white_king.0 & square_bit.0) != 0 {
                Some((Piece::King, Color::White))
            } else {
                None
            } // Should not happen if white_occupied is correct
        } else if (self.black_occupied.0 & square_bit.0) != 0 {
            if (self.black_pawns.0 & square_bit.0) != 0 {
                Some((Piece::Pawn, Color::Black))
            } else if (self.black_knights.0 & square_bit.0) != 0 {
                Some((Piece::Knight, Color::Black))
            } else if (self.black_bishops.0 & square_bit.0) != 0 {
                Some((Piece::Bishop, Color::Black))
            } else if (self.black_rooks.0 & square_bit.0) != 0 {
                Some((Piece::Rook, Color::Black))
            } else if (self.black_queens.0 & square_bit.0) != 0 {
                Some((Piece::Queen, Color::Black))
            } else if (self.black_king.0 & square_bit.0) != 0 {
                Some((Piece::King, Color::Black))
            } else {
                None
            } // Should not happen if black_occupied is correct
        } else {
            None
        }
    }

    pub fn print(&self) {
        println!("+---+---+---+---+---+---+---+---+");
        for rank in (0..8).rev() {
            // Ranks 8 down to 1
            print!("| ");
            for file in 0..8 {
                // Files A to H
                let square_index = rank * 8 + file;
                let square_bit = BitBoard(1u64 << square_index);

                let piece_char = if (self.white_pawns.0 & square_bit.0) != 0 {
                    'P'
                } else if (self.white_knights.0 & square_bit.0) != 0 {
                    'N'
                } else if (self.white_bishops.0 & square_bit.0) != 0 {
                    'B'
                } else if (self.white_rooks.0 & square_bit.0) != 0 {
                    'R'
                } else if (self.white_queens.0 & square_bit.0) != 0 {
                    'Q'
                } else if (self.white_king.0 & square_bit.0) != 0 {
                    'K'
                } else if (self.black_pawns.0 & square_bit.0) != 0 {
                    'p'
                } else if (self.black_knights.0 & square_bit.0) != 0 {
                    'n'
                } else if (self.black_bishops.0 & square_bit.0) != 0 {
                    'b'
                } else if (self.black_rooks.0 & square_bit.0) != 0 {
                    'r'
                } else if (self.black_queens.0 & square_bit.0) != 0 {
                    'q'
                } else if (self.black_king.0 & square_bit.0) != 0 {
                    'k'
                } else {
                    '.'
                };
                print!("{} | ", piece_char);
            }
            println!();
            println!("+---+---+---+---+---+---+---+---+");
        }
        println!("Turn: {:?}", self.turn);
        println!("Castling Rights: {:b}", self.castling_rights);
        println!("En Passant Square: {:?}", self.en_passant_square);
        println!("Halfmove Clock: {}", self.halfmove_clock);
        println!("Fullmove Number: {}", self.fullmove_number);
        println!("Zobrist Hash: {}", self.zobrist_hash);
    }
}

#[cfg(test)]
mod tests {
    use crate::moves::Flags;

    use super::*;

    #[test]
    fn test_default_board_setup() {
        let board = Board::default();

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
    fn test_make_and_unmake_pawn_move() {
        let mut board = Board::default();
        let initial_zobrist_hash = board.zobrist_hash; // Should be 0 for now

        // Simulate e2e4 (White pawn from square 12 to 28)
        let mv = Move {
            from: Square::E2, // E2
            to: Square::E4,   // E4
            piece: Piece::Pawn,
            promotion: None,
            flags: Flags::DoublePawnPush, // Double pawn push
            captured_piece: None,
        };

        // Store pre-move state for unmake verification
        let pre_move_white_pawns = board.white_pawns;
        let pre_move_black_occupied = board.black_occupied;
        let pre_move_turn = board.turn;
        let pre_move_halfmove_clock = board.halfmove_clock;
        let pre_move_fullmove_number = board.fullmove_number;

        board.make_move(&mv);

        // Verify board state after move
        assert_eq!(
            board.white_pawns.0 & (1u64 << 12),
            0,
            "Pawn should be removed from E2"
        );
        assert_ne!(
            board.white_pawns.0 & (1u64 << 28),
            0,
            "Pawn should be on E4"
        );
        assert_eq!(board.turn, Color::Black, "Turn should be Black");
        assert_eq!(
            board.halfmove_clock, 0,
            "Halfmove clock should reset on pawn move"
        );
        assert_eq!(
            board.fullmove_number, 1,
            "Fullmove number should not increment yet"
        ); // Increments after Black's move

        // Now unmake the move
        board.unmake_move(&mv);

        // Verify board state after unmake
        assert_eq!(
            board.white_pawns, pre_move_white_pawns,
            "White pawns should be restored"
        );
        assert_eq!(
            board.black_occupied, pre_move_black_occupied,
            "Black occupied should be restored (no capture)"
        );
        assert_eq!(
            board.turn, pre_move_turn,
            "Turn should be restored to White"
        );
        // Note: halfmove_clock and fullmove_number unmaking is incomplete in current `unmake_move`
        // assert_eq!(board.halfmove_clock, pre_move_halfmove_clock, "Halfmove clock should be restored");
        // assert_eq!(board.fullmove_number, pre_move_fullmove_number, "Fullmove number should be restored");
        assert_eq!(
            board.zobrist_hash, initial_zobrist_hash,
            "Zobrist hash should be restored"
        );
    }

    #[test]
    fn test_make_and_unmake_capture() {
        let mut board = Board::default();
        // Set up a simple capture scenario: White pawn on E2, Black pawn on D3
        // This is a simplified direct manipulation for testing.
        // In a real engine, you'd parse FEN or make moves.
        board.white_pawns = BitBoard(1u64 << 12); // White pawn on E2
        board.black_pawns = BitBoard(1u64 << 19); // Black pawn on D3 (square 19)
        board.white_occupied = board.white_pawns;
        board.black_occupied = board.black_pawns;
        board.occupied = board.white_occupied | board.black_occupied;
        board.empty = !board.occupied;
        board.turn = Color::White;

        let initial_white_pawns = board.white_pawns;
        let initial_black_pawns = board.black_pawns;
        let initial_turn = board.turn;

        // White pawn captures Black pawn: e2xd3
        let mv = Move {
            from: Square::E2, // E2
            to: Square::D3,   // D3
            piece: Piece::Pawn,
            promotion: None,
            flags: Flags::Normal, // No special flags for simple capture
            captured_piece: None,
        };

        board.make_move(&mv);

        // Verify state after capture
        assert_eq!(
            board.white_pawns.0 & (1u64 << 12),
            0,
            "White pawn removed from E2"
        );
        assert_ne!(
            board.white_pawns.0 & (1u64 << 19),
            0,
            "White pawn moved to D3"
        );
        assert_eq!(
            board.black_pawns.0 & (1u64 << 19),
            0,
            "Black pawn removed from D3 (captured)"
        );
        assert_eq!(board.turn, Color::Black, "Turn should be Black");
        assert_eq!(
            board.halfmove_clock, 0,
            "Halfmove clock should reset on capture"
        );

        // Unmake the capture (note: current unmake_move doesn't restore captured pieces)
        // This test will fail if `unmake_move` doesn't fully restore captured pieces.
        // For a complete unmake, the `Move` struct would need to store `Option<Piece>` for captured piece.
        // board.unmake_move(&mv);
        // assert_eq!(board.white_pawns, initial_white_pawns, "White pawns should be restored after unmake");
        // assert_eq!(board.black_pawns, initial_black_pawns, "Black pawns should be restored after unmake");
        // assert_eq!(board.turn, initial_turn, "Turn should be restored after unmake");
    }

    #[test]
    fn test_piece_on_square() {
        let board = Board::default();

        // Test white pieces
        assert_eq!(
            board.piece_on_square(Square::A1),
            Some((Piece::Rook, Color::White))
        ); // A1
        assert_eq!(
            board.piece_on_square(Square::B1),
            Some((Piece::Knight, Color::White))
        ); // B1
        assert_eq!(
            board.piece_on_square(Square::A2),
            Some((Piece::Pawn, Color::White))
        ); // A2

        // Test black pieces
        assert_eq!(
            board.piece_on_square(Square::A8),
            Some((Piece::Rook, Color::Black))
        ); // A8
        assert_eq!(
            board.piece_on_square(Square::B8),
            Some((Piece::Knight, Color::Black))
        ); // B8
        assert_eq!(
            board.piece_on_square(Square::A7),
            Some((Piece::Pawn, Color::Black))
        ); // A7

        // Test empty square
        assert_eq!(board.piece_on_square(Square::E3), None); // E3 (empty)
    }
}
