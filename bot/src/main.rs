use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use board::{Board, GameStatus};

const SCORE_LOSS: i32 = -10_000_000;
const INF: i32 = 2_000_000_000;
const COLUMN_WEIGHTS: [i32; 7] = [1, 2, 3, 5, 3, 2, 1];
const COL_MASK: u64 = 0b0000001_0000001_0000001_0000001_0000001_0000001;

pub fn evaluate(board: &Board) -> i32 {
    let mut my_score = 0;
    let mut opp_score = 0;

    let my_board = if board.red_turn { board.red } else { board.yellow };
    let opp_board = if board.red_turn { board.yellow } else { board.red };

    for col in 0..7 {
        let my_tokens_in_col = (my_board & (COL_MASK << (6 - col))).count_ones() as i32;
        let opp_tokens_in_col = (opp_board & (COL_MASK << (6 - col))).count_ones() as i32;

        my_score += my_tokens_in_col * COLUMN_WEIGHTS[col as usize];
        opp_score += opp_tokens_in_col * COLUMN_WEIGHTS[col as usize];
    }
    my_score - opp_score
}

fn negamax(board: &Board, depth: i32, mut alpha: i32, beta: i32) -> (i32, u8, i32) {
    let status = board.get_game_status();
    match status {
        GameStatus::Connected { red_win: _ } => return (SCORE_LOSS + board.nb_moves as i32, 0, 0),
        GameStatus::Draw => return (0, 0, 0),
        GameStatus::Ongoing => {}
    }

    if depth == 0 {
        return (evaluate(board), 0, 0);
    }

    let (nb_moves, moves) = board.generate_moves();
    if nb_moves == 0 { return (0, 0, 0); }
    
    let mut best_col = moves[0];
    let mut best_score = -INF;
    let mut sim = 0;
    
    for i in 0..nb_moves {
        let col = moves[i];
        let mut new_board = board.clone();
        new_board.make_move(col);
        
        let (mut score, _, nb_sim) = negamax(&new_board, depth - 1, -beta, -alpha);
        score = -score;
        sim += nb_sim + 1;
        
        if score > best_score {
            best_score = score;
            best_col = col;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    (best_score, best_col, sim)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth: i32 = if args.len() > 1 {
        args[1].parse().unwrap_or_else(|_| {
            println!("Argument invalide. Utilisation de la profondeur par défaut : 8");
            8
        })
    } else {
        8
    };

    println!("=== Lancement du Bot (Profondeur: {}) ===", depth);
    
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Arbitre introuvable !");
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    
    let mut local_board = Board::new();

    while let Ok(bytes_read) = reader.read_line(&mut buffer) {
        if bytes_read == 0 { break; }
        
        let message = buffer.trim();

        if message.starts_with("COLOR") {
            println!("Configuration : Je suis {}.", if message.contains("RED") { "ROUGE" } else { "JAUNE" });
        } 
        else if message.starts_with("BOARD") {
            if let Some(new_board) = Board::from_network_string(message) {
                local_board = new_board;
            }
        } 
        else if message == "PLAY" {
            let start = std::time::Instant::now();
            let (score, chosen_col, nb_sim) = negamax(&local_board, depth, -INF, INF);
            
            println!(
                "Je joue la colonne {} | Score: {} | Noeuds évalués: {} | Temps: {:?}", 
                chosen_col, score, nb_sim, start.elapsed()
            );
            
            writeln!(stream, "MOVE {}", chosen_col).expect("Échec de l'envoi");
        }
        buffer.clear();
    }
}