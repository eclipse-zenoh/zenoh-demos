mod common;
use common::{start_read_key_thread, start_tetris_thread, render_game_field};
use console::Term;
use human_hash::humanize;
use zenoh::{prelude::{Config, sync::SyncResolve, KeyExpr}, queryable::Query, sample::Sample};

fn main() {
    let term = Term::stdout();

    //
    // Prepare key expressions:
    // Queryable: tetris/{}
    // Publisher: tetris/{}/gamestate
    // Subscriber: tetris/{}/actions
    //
    let server_id = uuid::Uuid::new_v4();
    let server_name = humanize(&server_id, 1);
    let server_keyexpr= KeyExpr::new(format!("tetris/{}", server_id)).unwrap();
    let gamestate_keyexpr = server_keyexpr.join("gamestate").unwrap();
    let action_keyexpr = server_keyexpr.join("action").unwrap();

    let config = Config::default();
    let session = zenoh::open(config).res_sync().unwrap();

    //
    // Game discovery queryable
    //
    let discovery_callback = {
        let server_keyexpr = server_keyexpr.clone();
        let server_name =server_name.clone();
        move |query: Query| {
        let sample = Sample::new(server_keyexpr.clone(), server_name.clone());
        query.reply(Ok(sample)).res_sync().unwrap();
    }};
    let _queryable = session.declare_queryable(&server_keyexpr).callback(discovery_callback).res_sync().unwrap();

    //
    // Publisher for state of the game
    //
    let publisher = session.declare_publisher(&gamestate_keyexpr).res_sync().unwrap();

    //
    // Subscriber for actions from remote client
    // By default creates Receiver<Sample> which can be directly passed to the tetris thread
    //
    let subscriber = session.declare_subscriber(&action_keyexpr).res_sync().unwrap();

    let (action_rx_player, _) = start_read_key_thread();
    let state_rx = start_tetris_thread(
        action_rx_player,
        subscriber.receiver
    );

    term.clear_screen().unwrap();
    let text_player = vec!["PLAYER","", "<- Move Left", "-> Move Right", "^ Rotate", "v Accelerate", "Space: Drop"];
    let text_opponent = vec!["Server:", "", server_name.as_str()];

    while let Ok(state) = state_rx.recv() {
        let value = serde_json::to_string(&state).unwrap();
        publisher.put(value).res_sync().unwrap();
        render_game_field(&term, state, &text_player, &text_opponent);
    }
}