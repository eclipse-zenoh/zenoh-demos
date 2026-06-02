mod common;
use common::{start_read_key_thread, start_tetris_thread, render_game_field};
use console::Term;
use human_hash::humanize;
use zenoh::{Config, Wait};

fn main() {
    let term = Term::stdout();

    let server_id = uuid::Uuid::new_v4();
    let server_name = humanize(&server_id, 1);
    let server_keyexpr = format!("tetris/{}", server_id);
    let gamestate_keyexpr = format!("{}/gamestate", server_keyexpr);
    let action_keyexpr = format!("{}/action", server_keyexpr);

    let config = Config::default();
    let session = zenoh::open(config).wait().unwrap();

    // Game discovery queryable: reply with server name
    let _queryable = {
        let ke = server_keyexpr.clone();
        let name = server_name.clone();
        session
            .declare_queryable(&server_keyexpr)
            .callback(move |query| {
                query.reply(&ke, name.as_str()).wait().unwrap();
            })
            .wait()
            .unwrap()
    };

    let publisher = session
        .declare_publisher(&gamestate_keyexpr)
        .wait()
        .unwrap();

    let subscriber = session
        .declare_subscriber(&action_keyexpr)
        .wait()
        .unwrap();

    let (action_rx_player, _) = start_read_key_thread();
    let state_rx = start_tetris_thread(action_rx_player, subscriber);

    term.clear_screen().unwrap();
    let text_player = vec!["PLAYER","", "<- Move Left", "-> Move Right", "^ Rotate", "v Accelerate", "Space: Drop"];
    let text_opponent = vec!["Server:", "", server_name.as_str()];

    while let Ok(state) = state_rx.recv() {
        let value = serde_json::to_string(&state).unwrap();
        publisher.put(value).wait().unwrap();
        render_game_field(&term, state, &text_player, &text_opponent);
    }
}