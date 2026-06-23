use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use board::Board;

fn main() {
    println!("=== Lancement du Joueur Humain ===");
    
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("L'arbitre est introuvable sur le port 7878 !");
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    
    let mut local_board = Board::new();
    let mut _am_i_red: Option<bool> = None;

    println!("Connecté à l'arbitre. En attente...");

    while let Ok(bytes_read) = reader.read_line(&mut buffer) {
        if bytes_read == 0 {
            println!("L'arbitre a fermé la connexion.");
            break;
        }
        
        let message = buffer.trim();

        if message.starts_with("COLOR") {
            if message.contains("RED") {
                _am_i_red = Some(true);
                println!("🔴 Vous jouez les ROUGES.");
            } else if message.contains("YELLOW") {
                _am_i_red = Some(false);
                println!("🟡 Vous jouez les JAUNES.");
            }
        } 
        else if message.starts_with("BOARD") {
            if let Some(new_board) = Board::from_network_string(message) {
                local_board = new_board;
                println!("\n--- État du plateau ---");
                local_board.print_terminal(); 
            }
        } 
        else if message == "PLAY" {
            let (nb_moves, moves) = local_board.generate_moves();
            
            if nb_moves > 0 {
                let valid_moves = &moves[0..nb_moves];
                let mut chosen_col = None;

                // Boucle de saisie sécurisée
                while chosen_col.is_none() {
                    print!("C'est à vous ! Entrez une colonne {:?} : ", valid_moves);
                    io::stdout().flush().unwrap(); // Force l'affichage du print!
                    
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Erreur de lecture clavier");
                    
                    if let Ok(col) = input.trim().parse::<u8>() {
                        if valid_moves.contains(&col) {
                            chosen_col = Some(col);
                        } else {
                            println!("❌ Mouvement invalide ou colonne pleine.");
                        }
                    } else {
                        println!("❌ Veuillez entrer un chiffre valide.");
                    }
                }
                
                writeln!(stream, "MOVE {}", chosen_col.unwrap()).expect("Échec de l'envoi");
            }
        }

        buffer.clear();
    }
}