mod common;
use common::{start_read_key_thread, start_tetris_thread, render_game_field};
use console::Term;

fn main() {
    let term = Term::stdout();

    let (action_rx_player, action_rx_opponent) = start_read_key_thread();
    let state_rx = start_tetris_thread(
        action_rx_player,
        action_rx_opponent,
    );

    term.clear_screen().unwrap();
    let text_player = vec!["PLAYER","", "<- Move Left", "-> Move Right", "^ Rotate", "v Accelerate", "Space: Drop"];
    let text_opponent = vec!["OPPONENT","", "A: Move Left", "D: Move Right", "W: Rotate", "S: Accelerate", "Q: Drop"];

    while let Ok(state) = state_rx.recv() {
        render_game_field(&term, state, &text_player, &text_opponent);
    }
}