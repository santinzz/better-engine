use crate::board::{Board, Color};

#[derive(Debug, PartialEq)]
pub enum GameResult {
    Checkmate(Color),       // side-to-move was mated
    Stalemate,              // no legal moves but not in check
    DrawFiftyMove,          // halfmove_clock >= 100
    DrawRepetition,         // same position occurred 3 times
    DrawInsufficientMaterial,
    Ongoing,
}

impl Board {
  pub fn game_result(&self) -> GameResult {
    let legal = self.generate_legal_moves();

    if legal.is_empty() {
      let king_sq = if self.turn == Color::White {
        self.white_king.next_square()
      } else {
        self.black_king.next_square()
      };

      if let Some(king_sq) = king_sq {
        if self.is_square_attacked(king_sq, self.turn.opposite()) {
          return GameResult::Checkmate(self.turn);
        } else {
          return GameResult::Stalemate
        }
      }
    }

    if self.halfmove_clock >= 100 {
      return GameResult::DrawFiftyMove;
    }

    if self.is_insufficient_material() {
      return GameResult::DrawInsufficientMaterial;
    }

    GameResult::Ongoing
  }
}