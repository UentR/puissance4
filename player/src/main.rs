use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use board::Board; // Assure-toi que ton Cargo.toml de 'player' a bien 'board' en dépendance

fn main() {
    println!("Démarrage du processus joueur...");
    
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("L'arbitre est introuvable sur le port 7878 !");
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    
    // État local du joueur
    let mut local_board = Board::new();
    let mut am_i_red: Option<bool> = None;

    println!("Connecté à l'arbitre. En attente de configuration...");

    // Boucle d'écoute principale
    while let Ok(bytes_read) = reader.read_line(&mut buffer) {
        if bytes_read == 0 {
            println!("L'arbitre a fermé la connexion. Fin du programme.");
            break;
        }
        
        let message = buffer.trim();

        // 1. L'arbitre nous donne notre couleur
        if message.starts_with("COLOR") {
            if message.contains("RED") {
                am_i_red = Some(true);
                println!("✅ Configuration reçue : Je suis le joueur ROUGE.");
            } else if message.contains("YELLOW") {
                am_i_red = Some(false);
                println!("✅ Configuration reçue : Je suis le joueur JAUNE.");
            }
        } 
        // 2. L'arbitre synchronise le plateau
        else if message.starts_with("BOARD") {
            if let Some(new_board) = Board::from_network_string(message) {
                local_board = new_board;
                // Décommenter la ligne suivante si tu veux afficher la grille à chaque tour
                local_board.print_terminal(); 
            } else {
                eprintln!("Erreur : Impossible de parser le message BOARD reçu.");
            }
        } 
        // 3. L'arbitre nous donne l'ordre de jouer
        else if message == "PLAY" {
            println!("C'est à mon tour de jouer !");
            
            // On génère les coups légaux à partir de notre copie locale du plateau
            let (nb_moves, moves) = local_board.generate_moves();
            
            if nb_moves > 0 {
                // Pour l'instant, le bot est naïf : il prend le premier coup disponible.
                // (Comme MOVE_ORDER teste le centre en premier, il jouera au centre si c'est libre).
                let chosen_move = moves[0];
                
                println!("Je choisis la colonne {}", chosen_move);
                
                // On envoie notre décision à l'arbitre
                writeln!(stream, "MOVE {}", chosen_move).expect("Échec de l'envoi du coup");
            } else {
                eprintln!("Erreur fatale : L'arbitre me demande de jouer mais la grille est pleine !");
            }
        }

        // Toujours nettoyer le buffer pour la prochaine ligne TCP
        buffer.clear();
    }
}