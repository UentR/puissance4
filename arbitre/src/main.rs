use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use board::Board;
use board::GameStatus;

use rand::rng;
use rand::RngExt;

fn check_speed() {
    println!("=== Démarrage de l'Arbitre ===");

    let mut my_rng = rand::rng();

    let start = std::time::Instant::now();

    let nb_games = 100_000_000;
    let mut nb_moves = 0;
    let mut game_board = Board::new();
    for _i in 1..=nb_games {
        game_board.reset();
        loop {
            let (nb_moves_valid, moves) = game_board.generate_moves();

            // game_board.print_terminal();
            // println!("Nombre de coups valides : {}", nb_moves_valid);
            // for i in 0..nb_moves_valid {
            //     println!("Coup valide {} : {}", i, moves[i]);
            // }

            // generate random int from 0 to nb_moves_valid - 1
            let idx = my_rng.random_range(0..nb_moves_valid);
            // println!("Coup choisi aléatoirement : {}", moves[idx]);
            let chosen_move = moves[idx];
            nb_moves += 1;

            game_board.make_move(chosen_move);

            let status = game_board.get_game_status();
            if status != GameStatus::Ongoing {
                break;
            }
        }
    }

    println!("\n==============================");
    println!(" Simulation terminée ! ");
    println!(" Nombre de parties simulées : {}", nb_games);
    println!(" Nombre de coups joués : {}", nb_moves);
    println!(" Temps écoulé : {:?}", start.elapsed());
    println!(" Temps moyen par partie : {:?}", start.elapsed() / nb_games);
    println!(" Temps moyen par coup : {:?}", start.elapsed() / nb_moves);

    println!("==============================");
}

fn start_game() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Le port 7878 est déjà utilisé");
    println!("Arbitre démarré sur 127.0.0.1:7878.");

    println!("En attente du Joueur 1 (Rouge)...");
    let (mut p1_socket, p1_addr) = listener.accept().expect("Erreur connexion J1");
    println!("Joueur 1 connecté depuis : {}", p1_addr);
    let mut p1_reader = BufReader::new(p1_socket.try_clone().unwrap());
    writeln!(p1_socket, "COLOR RED").unwrap();

    println!("En attente du Joueur 2 (Jaune)...");
    let (mut p2_socket, p2_addr) = listener.accept().expect("Erreur connexion J2");
    println!("Joueur 2 connecté depuis : {}", p2_addr);
    let mut p2_reader = BufReader::new(p2_socket.try_clone().unwrap());
    writeln!(p2_socket, "COLOR YELLOW").unwrap();

    let mut game_board = Board::new();
    println!("\n=== DÉBUT DE LA PARTIE ===");

    loop {
        let board_msg = game_board.to_network_string();
        writeln!(p1_socket, "{}", board_msg).expect("Erreur d'envoi J1");
        writeln!(p2_socket, "{}", board_msg).expect("Erreur d'envoi J2");

        let (active_socket, active_reader, nom_joueur) = if game_board.red_turn {
            (&mut p1_socket, &mut p1_reader, "Joueur 1 (Rouge)")
        } else {
            (&mut p2_socket, &mut p2_reader, "Joueur 2 (Jaune)")
        };

        println!("Au tour de {} de jouer.", nom_joueur);

        writeln!(active_socket, "PLAY").expect("Erreur d'envoi PLAY");

        let mut buffer = String::new();
        if active_reader.read_line(&mut buffer).is_err() || buffer.is_empty() {
            println!("Connexion perdue avec {}. Fin de la partie.", nom_joueur);
            break;
        }

        let message = buffer.trim();
        println!("{} a répondu : {}", nom_joueur, message);

        if message.starts_with("MOVE ") {
            // On tente d'extraire le numéro de la colonne
            if let Ok(col) = message[5..].trim().parse::<u8>() {
                
                // On vérifie si le coup est valide sur le plateau
                if game_board.make_move(col) {
                    println!("Coup validé !");
                } else {
                    println!("ERREUR : {} a tenté un coup invalide (colonne {}). On redemande.", nom_joueur, col);
                    // Le `continue` permet de recommencer la boucle sans changer de tour
                    // (red_turn n'a pas basculé puisque make_move a renvoyé false)
                    continue; 
                }
            } else {
                println!("Commande MOVE mal formatée. Attendu: 'MOVE X'");
                continue;
            }
        } else {
            println!("Commande inconnue reçue.");
            continue;
        }

        // 5. Vérification des conditions de victoire
        let status = game_board.get_game_status();
        if status != GameStatus::Ongoing {
            // Envoi de l'état final du plateau pour affichage
            let final_board = game_board.to_network_string();
            let _ = writeln!(p1_socket, "{}", final_board);
            let _ = writeln!(p2_socket, "{}", final_board);

            match status {
                GameStatus::Connected { red_win: true } => println!("🏆 VICTOIRE DU JOUEUR 1 (ROUGE) !"),
                GameStatus::Connected { red_win: false } => println!("🏆 VICTOIRE DU JOUEUR 2 (JAUNE) !"),
                GameStatus::Draw => println!("🤝 MATCH NUL ! Le plateau est plein."),
                _ => {} // Ne devrait jamais arriver grâce au if
            }
            break; // On sort de la boucle, la partie est finie
        }
    }
}

fn main() {
    start_game();
    // check_speed();
}