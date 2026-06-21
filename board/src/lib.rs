#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Connected { red_win: bool },
    Draw,
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub red: u64,
    pub yellow: u64,

    pub nb_moves: u8,
    pub red_turn: bool,
}

const WIDTH: u8 = 7;
const HEIGHT: u8 = 6;
const COL_MASK: u64 = 0b0000001_0000001_0000001_0000001_0000001_0000001;
const FULL_BOARD: u64 = (1 << (WIDTH * HEIGHT)) - 1;

impl Board {
    pub fn new() -> Self {
        Board {
            red: 0,
            yellow: 0,
            nb_moves: 0,
            red_turn: true,
        }
    }

    pub fn reset(&mut self) {
        self.red = 0;
        self.yellow = 0;
        self.nb_moves = 0;
        self.red_turn = true;
    }

    const MOVE_ORDER: [u8; 7] = [3, 2, 4, 1, 5, 0, 6];

    #[inline]
    pub fn generate_moves(&self) -> (usize, [u8; 7]) {
        let state: u64 = self.red | self.yellow;
        

        let top_row = (state >> 35) & 0b1111111;
        

        let valid_cols = (!top_row) & 0b1111111;

        let mut moves: [u8; 7] = [0; 7];
        let mut count = 0;


        for &col in &Self::MOVE_ORDER {
            if (valid_cols & (1 << col)) != 0 {
                moves[count] = col+1;
                count += 1;
            }
        }

        (count, moves)
    }

    pub fn make_move(&mut self, col: u8) -> bool {
        let state: u64 = (self.red | self.yellow) & FULL_BOARD;

        if (col < 1) || (col > WIDTH) {
            return false;
        }

        let mask: u64 = COL_MASK << (7-col);
        
        let curr_col_mask: u64 = (!state)&mask;
        let max_row: u64 = ((curr_col_mask << WIDTH) ^ curr_col_mask) & FULL_BOARD;
        let idx = max_row.trailing_zeros();
        
        if self.red_turn {
            self.red |= 1 << idx;
            self.red &= FULL_BOARD;
        } else {
            self.yellow |= 1 << idx;
            self.yellow &= FULL_BOARD;
        }


        self.nb_moves += 1;
        self.red_turn = !self.red_turn;

        true
    }

    pub fn print_terminal(&self) {
        println!("Nombre de coups joués : {}", self.nb_moves);
        for i in (0..(WIDTH*HEIGHT)).rev() {
            let mask: u64 = 1 << i;
            if self.red & mask > 0 {
                print!("R ");
            } else if self.yellow & mask > 0 {
                print!("Y ");
            } else {
                print!(". ");
            }

            if i % WIDTH == 0 {
                print!("\n");
            }
        }
        print!("\n");
    }

    #[inline]
    pub fn has_won(player_board: u64) -> bool {
        const MASK_RIGHT: u64 = 0b0001111_0001111_0001111_0001111_0001111_0001111;
        const MASK_LEFT: u64  = 0b1111000_1111000_1111000_1111000_1111000_1111000;

        let m = player_board & (player_board >> 1);
        if (m & (m >> 2) & MASK_RIGHT) != 0 { return true; }

        let m = player_board & (player_board >> 7);
        if (m & (m >> 14)) != 0 { return true; }

        let m = player_board & (player_board >> 8);
        if (m & (m >> 16) & MASK_RIGHT) != 0 { return true; }

        let m = player_board & (player_board >> 6);
        if (m & (m >> 12) & MASK_LEFT) != 0 { return true; }

        false
    }


    pub fn get_game_status(&self) -> GameStatus {
        if Self::has_won(self.red) {
            return GameStatus::Connected { red_win: true };
        }
        if Self::has_won(self.yellow) {
            return GameStatus::Connected { red_win: false };
        }
        
        if self.nb_moves >= 42 {
            return GameStatus::Draw; 
        }
        
        GameStatus::Ongoing
    }

    pub fn to_network_string(&self) -> String {
        format!(
            "BOARD {} {} {} {}",
            self.red,
            self.yellow,
            self.nb_moves,
            if self.red_turn { 1 } else { 0 }
        )
    }

    
    pub fn from_network_string(data: &str) -> Option<Self> {
        
        let parts: Vec<&str> = data.trim().split_whitespace().collect();
        
        
        if parts.len() == 5 && parts[0] == "BOARD" {
            let red = parts[1].parse::<u64>().ok()?;
            let yellow = parts[2].parse::<u64>().ok()?;
            let nb_moves = parts[3].parse::<u8>().ok()?;
            let red_turn = parts[4].parse::<u8>().ok()? == 1;
            
            Some(Board { red, yellow, nb_moves, red_turn })
        } else {
            None
        }
    }
}