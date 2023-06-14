mod common;
use std::{sync::Arc, thread};

use common::{render_game_field, key_to_action_player};
use console::Term;
use gametetris_rs::TetrisPairState;
use zenoh::{prelude::{Config, sync::SyncResolve, KeyExpr}, Session};

fn start_publish_action_thread(session: Arc<Session>, action_keyexpr: KeyExpr) {
    let term = Term::stdout();
    let action_keyexpr = action_keyexpr.clone().into_owned();
    thread::spawn(move || {
        let publisher = session
            .declare_publisher(&action_keyexpr)
            .res_sync()
            .unwrap();
        loop {
            let key = term.read_key().unwrap();
            if let Some(action) = key_to_action_player(&key) {
                let value = serde_json::to_string(&action).unwrap();
                publisher.put(value).res_sync().unwrap();
            }
        }
    });
}

fn main() {
    let term = Term::stdout();

    let config = Config::default();
    let session = Arc::new(zenoh::open(config).res_sync().unwrap());

    //
    // Find available servers and select one
    //
    let receiver = session.get("tetris/*").res_sync().unwrap();
    let mut servers = Vec::new();
    while let Ok(reply) = receiver.recv() {
        if let Ok(sample) = reply.sample {
            servers.push((sample.key_expr.to_string(), sample.value.to_string()));
        }
    }
    if servers.is_empty() {
        println!("No servers found");
        return;
    }
    println!("Select server:");
    servers.insert(0, ("tetris/*".to_string(), "ALL".to_string()));
    (0..servers.len()).for_each(|n| {
        println!("{}: {} at {}", n, servers[n].0, servers[n].1)
    });
    let n = loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let n = line.trim().parse::<usize>().unwrap();
        if n < servers.len() {
            break n;
        }
    };
    let server_keyexpr = KeyExpr::new(&servers[n].0).unwrap();
    let server_name = &servers[n].1;
    println!("Selected server: {} at {}", server_name, server_keyexpr);

    //
    // Declare a subscriber to receive game state from the server
    //
    let gamestate_keyexpr = server_keyexpr.join("gamestate").unwrap();
    let subscriber = session
        .declare_subscriber(&gamestate_keyexpr)
        .res_sync()
        .unwrap();

    //
    // Read keys from console and send them to the server in separate thread
    //
    let action_keyexpr = server_keyexpr.join("action").unwrap();
    start_publish_action_thread(session.clone(), action_keyexpr);

    //
    // Receive game state from the server and render it
    //
    let text_opponent = vec![server_name.as_str()];
    let text_player = vec!["PLAYER","", "<- Move Left", "-> Move Right", "^ Rotate", "v Accelerate", "Space: Drop"];
    while let Ok(sample) = subscriber.recv() {
        let mut state: TetrisPairState =
            serde_json::from_str(sample.value.to_string().as_str()).unwrap();
        state.swap();
        render_game_field(&term, state, &text_player, &text_opponent);
    }
}