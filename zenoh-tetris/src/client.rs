mod common;
use std::{sync::Arc, thread};

use common::{render_game_field, key_to_action_player};
use console::Term;
use gametetris_rs::TetrisPairState;
use zenoh::{Config, Wait};

fn start_publish_action_thread(session: Arc<zenoh::Session>, action_keyexpr: String) {
    let term = Term::stdout();
    thread::spawn(move || {
        let publisher = session
            .declare_publisher(&action_keyexpr)
            .wait()
            .unwrap();
        loop {
            let key = term.read_key().unwrap();
            if let Some(action) = key_to_action_player(&key) {
                let value = serde_json::to_string(&action).unwrap();
                publisher.put(value).wait().unwrap();
            }
        }
    });
}

fn main() {
    let term = Term::stdout();

    let config = Config::default();
    let session = Arc::new(zenoh::open(config).wait().unwrap());

    // Find available servers and select one
    let receiver = session.get("tetris/*").wait().unwrap();
    let mut servers = Vec::new();
    while let Ok(reply) = receiver.recv() {
        if let Ok(sample) = reply.into_result() {
            let key = sample.key_expr().to_string();
            let name = String::from_utf8_lossy(&sample.payload().to_bytes()).to_string();
            servers.push((key, name));
        }
    }
    if servers.is_empty() {
        println!("No servers found");
        return;
    }
    println!("Select server:");
    servers.insert(0, ("tetris/*".to_string(), "ALL".to_string()));
    (0..servers.len()).for_each(|n| {
        println!("{}: {} at {}", n, servers[n].1, servers[n].0)
    });
    let n = loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let n = line.trim().parse::<usize>().unwrap();
        if n < servers.len() {
            break n;
        }
    };
    let server_keyexpr = servers[n].0.clone();
    let server_name = servers[n].1.clone();
    println!("Selected server: {} at {}", server_name, server_keyexpr);

    // Subscribe to game state from server
    let gamestate_keyexpr = format!("{}/gamestate", server_keyexpr);
    let subscriber = session
        .declare_subscriber(&gamestate_keyexpr)
        .wait()
        .unwrap();

    // Read keys and send actions to server in a separate thread
    let action_keyexpr = format!("{}/action", server_keyexpr);
    start_publish_action_thread(session.clone(), action_keyexpr);

    let text_opponent = vec![server_name.as_str()];
    let text_player = vec!["PLAYER","", "<- Move Left", "-> Move Right", "^ Rotate", "v Accelerate", "Space: Drop"];
    while let Ok(sample) = subscriber.recv() {
        let mut state: TetrisPairState =
            serde_json::from_str(
                &String::from_utf8_lossy(&sample.payload().to_bytes())
            ).unwrap();
        state.swap();
        render_game_field(&term, state, &text_player, &text_opponent);
    }
}