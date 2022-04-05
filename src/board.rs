use std::cmp::Ordering::{Less, Greater};
use std::cmp::max;
use itertools::Itertools;
use types::*;
use util::*;
use _move::*;
use table::Hash;
use bitboard::BitBoard;
use print::*;
use magics::*;

pub const START_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub type Squares = [u8; 64];

#[derive(Copy)]
pub struct Board {
    pub bb: BitBoard,
    pub sqs: Squares,
    pub ply: usize,
    pub to_move: u8,
    pub hash: Hash,
    pub castling: u8,
    pub en_passant: u64
}

impl Clone for Board { fn clone(&self) -> Self { *self } }

impl Board {
    pub fn start_position() -> Self {
        Board::from_fen(&mut START_FEN.split_whitespace())
    }

    pub fn from_fen(fen: &mut Params) -> Self {
        let fen_board = fen.next().expect("Missing fen board");
        let reversed_rows = fen_board.split('/').rev(); // fen is read from top row

        let mut sqs = [EMPTY; 64];

        for (r, row) in reversed_rows.enumerate() {
            let mut offset = 0;
            for (c, ch) in row.chars().enumerate() {
                if !ch.is_numeric() {
                    sqs[r*8 + c+offset] = to_piece(ch);
                } else {
                    offset += (ch as u8 - b'1') as usize;
                }
            }
        }

        let to_move = match fen.next().expect("[w, b]") {
            "w" => WHITE,
            _   => BLACK,
        };

        let castle_str = fen.next().expect("Castling [KQkq]");
        let mut castling = 0;
        if castle_str.contains('K') { castling |= WK_CASTLE }
        if castle_str.contains('Q') { castling |= WQ_CASTLE }
        if castle_str.contains('k') { castling |= BK_CASTLE }
        if castle_str.contains('q') { castling |= BQ_CASTLE }

        let ep_sq: Vec<char> = fen.next().expect("En Passant target square").chars().collect();

        let en_passant = match ep_sq[..] {
            ['-'] => 0,
            [sc @ 'a'..='h', sr @ '3', '6'] => 1 << to_pos(sc, sr),
            _ => panic!("Invalid en passant token")
        };

        let bitboard = BitBoard::generate_from(&sqs);
        let hash = Hash::init(&sqs, castling, en_passant, to_move);

        Board { bb: bitboard, sqs: sqs, ply: 0, to_move: to_move,
                hash: hash, castling: castling, en_passant: en_passant }
    }

    pub fn perft(&self, depth: u8, print: bool) -> usize {
        if self.player_in_check(self.prev_move()) { return 0 }

        if depth == 0 { return 1 }

        let mut count = 0;
        for mv in self.get_moves() {
            let mut new_board = *self;
            new_board.make_move(mv);

            let n = new_board.perft(depth - 1, false);
            if print && n > 0 { println!("{}: {}", mv, n) }
            count += n;
        }
        count
    }

    pub fn do_null_move(&mut self) {
        self.ply += 1;
        self.to_move = flip(self.to_move);
        self.hash.flip_color();
        self.hash.set_ep(self.en_passant);
        self.en_passant = 0;
    }

    /// Move the specified piece, which may not be the original src piece (when promoting)
    /// Update the board hash correspondingly
    pub fn move_piece(&mut self, src: usize, dest: usize, piece: u8) {
        let (src_pc, dest_pc) = (self.sqs[src], self.sqs[dest]);

        self.hash.set_piece(src, src_pc); // Remove moving piece
        self.bb[src_pc] ^= 1 << src;
        if dest_pc != EMPTY {
            self.hash.set_piece(dest, dest_pc); // Remove destination piece
            self.bb[dest_pc] ^= 1 << dest;
        }

        self.sqs[src]  = EMPTY;
        self.sqs[dest] = piece;

        self.hash.set_piece(dest, piece); // Add src piece at dest square
        self.bb[piece] ^= 1 << dest;
    }

    /// Toggle the state of one individual castling option
    pub fn set_castle(&mut self, castle: u8) {
        self.hash.set_castling(self.castling); // Remove castling state
        self.castling ^= castle;
        self.hash.set_castling(self.castling); // Add new state to hash
    }

    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        let color = self.to_move;
        let opp = flip(color);
        let offset = Board::color_offset(color);

        self.do_null_move();

        let dest_piece = match mv.promotion() {
            QUEEN_PROM  => QUEEN  | color,
            ROOK_PROM   => ROOK   | color,
            BISHOP_PROM => BISHOP | color,
            KNIGHT_PROM => KNIGHT | color,
            _ => self.sqs[src] // If there is no promotion
        };
        self.move_piece(src, dest, dest_piece);

        if mv.king_castle() {
            self.move_piece(7 + offset, 5 + offset, ROOK | color);
        }

        if mv.queen_castle() {
            self.move_piece(offset, 3 + offset, ROOK | color);
        }

        if mv.is_en_passant() {
            // If white takes -> remove from row below. If black takes -> remove from row above
            let ep_pawn = if color == WHITE { dest - 8 } else { dest + 8 };
            self.hash.set_piece(ep_pawn, self.sqs[ep_pawn]); // Remove taken pawn
            self.bb[PAWN | opp] ^= 1 << ep_pawn;
            self.sqs[ep_pawn] = EMPTY;
        }

        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
            self.hash.set_ep(self.en_passant);
        }

        if  self.castling & (BK_CASTLE << color) != 0 &&
            (src == offset + 4 || src == offset + 7) {
                self.set_castle(BK_CASTLE << color);
        }

        if  self.castling & (BK_CASTLE << opp) != 0 &&
            dest == Board::color_offset(opp) + 7 {
                self.set_castle(BK_CASTLE << opp);
        }

        if  self.castling & (BQ_CASTLE << color) != 0 &&
            (src == offset + 4 || src == offset) {
                self.set_castle(BQ_CASTLE << color);
        }

        if  self.castling & (BQ_CASTLE << opp) != 0 &&
            dest == Board::color_offset(opp) {
                self.set_castle(BQ_CASTLE << opp);
        }

        self.bb.set_all();
    }

    /// Get the appropriate offset for castling depending on color to move
    pub fn color_offset(us: u8) -> usize {
        if us == WHITE { 0 } else { 56 }
    }

    pub fn move_from_str(&mut self, mv: &str) -> Move {
        let moves: Vec<char> = mv.chars().collect();
        if let [sc, sr, dc, dr, ref promotion @ ..] = moves[..] {
            let (src, dest) = (to_pos(sc, sr), to_pos(dc, dr));

            let mut flags = match promotion {
                &['q'] => QUEEN_PROM,
                &['r'] => ROOK_PROM,
                &['b'] => BISHOP_PROM,
                &['n'] => KNIGHT_PROM,
                _ => 0
            };

            flags |= match mv {
                "e1g1" if self.castling & WK_CASTLE != 0 => CASTLES_KING,
                "e8g8" if self.castling & BK_CASTLE != 0 => CASTLES_KING,
                "e1c1" if self.castling & WQ_CASTLE != 0 => CASTLES_QUEEN,
                "e8c8" if self.castling & BQ_CASTLE != 0 => CASTLES_QUEEN,
                _ => 0
            };

            if self.sqs[src as usize] & PIECE == PAWN {
                let is_double = if src > dest { src-dest } else { dest-src } == 16;
                if is_double { flags |= DOUBLE_PAWN_PUSH };

                let is_en_passant = dest == lsb(self.en_passant);
                if is_en_passant { flags |= EN_PASSANT };
            }

            Move::new(src, dest, flags)
        } else {
            panic!("Malformed move {}", mv)
        }
    }

    pub fn see(&mut self, pos: u32, us: u8) -> i32 {
        let captured = self.sqs[pos as usize];
        let (attacker, from) = self.attacker(pos, us);

        if attacker != EMPTY {
            self.move_piece(from as usize, pos as usize, attacker);
            max(0, p_val(captured) as i32 - self.see(pos, flip(us)))
        } else {
            0
        }
    }

    pub fn see_move(&self, mv: &Move) -> i32 {
        if !mv.is_capture() { return 0 }
        let captured = self.sqs[mv.to() as usize];
        let mut clone  = *self;

        clone.make_move(*mv);
        p_val(captured) as i32 - clone.see(mv.to(), self.to_move)
    }

    /// Return the lowest valued enemy attacker of a given square and the attackers position
    pub fn attacker(&self, pos: u32, us: u8) -> (u8, u32) {
        let bb = &self.bb;
        let opp = flip(us);

        let l_file = if pos % 8 > 0 { file(pos - 1) } else { 0 };
        let r_file = if pos % 8 < 7 { file(pos + 1) } else { 0 };
        let row_n = pos / 8;
        let attacking_row = match opp {
            WHITE if row_n > 1 => row(pos - 8),
            BLACK if row_n < 6 => row(pos + 8),
            _ => 0
        };

        let pawns = bb[PAWN | opp] & attacking_row & (l_file | r_file);
        if pawns != 0 { return (PAWN | opp, lsb(pawns)) }

        let knights = knight_moves(pos) & bb[KNIGHT | opp];
        if knights != 0 { return (KNIGHT | opp, lsb(knights)) }

        let occ = bb[ALL | us] | bb[ALL | opp];

        let bishops = bishop_moves(pos, occ) & bb[BISHOP | opp];
        if bishops != 0 { return (BISHOP | opp, lsb(bishops)) }

        let rooks = rook_moves(pos, occ) & bb[ROOK | opp];
        if rooks != 0 { return (ROOK | opp, lsb(rooks)) }

        let queens = queen_moves(pos, occ) & bb[QUEEN | opp];
        if queens != 0 { return (QUEEN | opp, lsb(queens)) }

        let king = king_moves(pos) & bb[KING | opp];
        if king != 0 { return (KING | opp, lsb(king)) }

        (EMPTY, !0)
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let bb = &self.bb;
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let (us, opp) = (self.to_move, self.prev_move());
        let enemies = bb[ALL | opp];
        let occ = bb[ALL | us] | enemies;

        let (row_3, row_8, l_file, r_file, up, left, right) =
            if us == WHITE { PAWN_INFO_WHITE } else { PAWN_INFO_BLACK };

        for_all(bb[QUEEN | us], &mut |from| {
            let mvs = queen_moves(from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[ROOK | us], &mut |from| {
            let mvs = rook_moves(from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[BISHOP | us], &mut |from| {
            let mvs = bishop_moves(from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[KNIGHT | us], &mut |from| {
            let mvs = knight_moves(from);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        let from = lsb(bb[KING | us]);
        let mvs = king_moves(from);
        add_moves_from(&mut moves, from, mvs & !occ, 0);
        add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);

        let (pushes, double_pushes, left_attacks, right_attacks);
        let pawns = bb[PAWN | us];

        if us == WHITE {
            pushes = (pawns << up) & !occ;
            double_pushes = ((pushes & row_3) << up) & !occ;
            left_attacks = (pawns << left) & (enemies | self.en_passant) & !r_file;
            right_attacks = (pawns << right) & (enemies | self.en_passant) & !l_file;
        } else {
            pushes = (pawns >> -up) & !occ;
            double_pushes = ((pushes & row_3) >> -up) & !occ;
            left_attacks = (pawns >> -left) & (enemies | self.en_passant) & !r_file;
            right_attacks = (pawns >> -right) & (enemies | self.en_passant) & !l_file;
        }

        let l_en_passant = left_attacks & self.en_passant;
        let r_en_passant = right_attacks & self.en_passant;
        let prom_pushes = pushes & row_8;
        let prom_l_att = left_attacks & row_8;
        let prom_r_att = right_attacks & row_8;

        add_moves(&mut moves, pushes ^ prom_pushes, up, 0);
        add_moves(&mut moves, double_pushes, up+up, DOUBLE_PAWN_PUSH);
        add_moves(&mut moves, left_attacks ^ l_en_passant ^ prom_l_att, left, IS_CAPTURE);
        add_moves(&mut moves, right_attacks ^ r_en_passant ^ prom_r_att, right, IS_CAPTURE);
        add_moves(&mut moves, l_en_passant, left, EN_PASSANT | IS_CAPTURE);
        add_moves(&mut moves, r_en_passant, right, EN_PASSANT | IS_CAPTURE);
        add_prom_moves(&mut moves, prom_pushes, up, 0);
        add_prom_moves(&mut moves, prom_l_att, left, IS_CAPTURE);
        add_prom_moves(&mut moves, prom_r_att, right, IS_CAPTURE);

        let offset = Board::color_offset(us);

        if    self.castling & (BK_CASTLE << us) != 0
           && self.sqs[offset + 5] == EMPTY
           && self.sqs[offset + 6] == EMPTY
           && self.attacker(offset as u32 + 5, us).0 == EMPTY
           && self.attacker(offset as u32 + 6, us).0 == EMPTY
           && self.attacker(offset as u32 + 4, us).0 == EMPTY
        {
            add_moves(&mut moves, 1 << (offset + 6), 2, CASTLES_KING);
        }

        if    self.castling & (BQ_CASTLE << us) != 0
           && self.sqs[offset + 3] == EMPTY
           && self.sqs[offset + 2] == EMPTY
           && self.sqs[offset + 1] == EMPTY
           && self.attacker(offset as u32 + 3, us).0 == EMPTY
           && self.attacker(offset as u32 + 2, us).0 == EMPTY
           && self.attacker(offset as u32 + 4, us).0 == EMPTY
        {
            add_moves(&mut moves, 1 << (offset + 2), -2, CASTLES_QUEEN);
        }
        // TODO: moves.shrink_to_fit()
        moves
    }

    /// Move better SEE to the front to improve move ordering in alpha-beta search
    pub fn qsort(&self, moves: &[Move]) -> Vec<(i32, Move)> {
        moves.iter().map(
            |mv| (self.see_move(mv), *mv))
            .filter(|e| e.0 > 0)
            .sorted_by(|a,b| if a.0 > b.0 { Less } else { Greater })
    }

    pub fn sort_with(&self, moves: Vec<Move>, best: Move, killer: &Killer) -> Vec<(i32, Move)> {
        moves.iter().map(
            |&mv| {
                // Give the largest value to the best move to place it at the front
                // Give killer moves values just above zero to put them ahead of
                // all non-captures and behind all positive see moves
                let see = if mv == best     { 10_000_000 }
                     else if mv == killer.0 { 2 }
                     else if mv == killer.1 { 1 }
                     else { self.see_move(&mv) };

                (see, mv)
            })
            .sorted_by(|a,b| if a.0 > b.0 { Less } else { Greater })
    }

    pub fn player_in_check(&self, us: u8) -> bool {
        let king_pos = lsb(self.bb[KING | us]);
        self.attacker(king_pos, us).0 != EMPTY
    }

    pub fn is_irreversible(&self, mv: Move) -> bool {
           mv.is_capture()
        || self.sqs[mv.from() as usize] & PIECE == PAWN
        || mv.king_castle()
        || mv.queen_castle()
    }

    pub fn is_in_check(&self) -> bool {
        self.player_in_check(self.to_move)
    }

    pub fn is_white(&self) -> bool {
        self.to_move == WHITE
    }

    pub fn prev_move(&self) -> u8 {
        flip(self.to_move)
    }
}
