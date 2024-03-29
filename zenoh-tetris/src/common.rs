use std::time;
use std::thread;

use console::Key;
use console::Term;
use flume::Receiver;
use gametetris_rs::AnsiTermStyle;
use gametetris_rs::GameFieldPair;
use gametetris_rs::PlayerSide;
use gametetris_rs::StepResult;
use gametetris_rs::TermRender;
use gametetris_rs::{Action, TetrisPairState, TetrisPair};
use zenoh::sample::Sample;

pub struct TetrisThreadAction(Action);

impl TryFrom<Action> for TetrisThreadAction {
    type Error = ();
    fn try_from(value: Action) -> Result<Self, Self::Error> {
        Ok(TetrisThreadAction(value))
    }
}

impl TryFrom<Sample> for TetrisThreadAction {
    type Error = ();
    fn try_from(value: Sample) -> Result<Self, Self::Error> {
        let s = value.value.to_string();
        let action = serde_json::from_str(s.as_str());
        match action {
            Ok(action) => Ok(TetrisThreadAction(action)),
            Err(_) => Err(()),
        }
    }
}

#[allow(dead_code)]
pub fn start_tetris_thread<
    T1: TryInto<TetrisThreadAction, Error = ()> + Send + 'static,
    T2: TryInto<TetrisThreadAction, Error = ()> + Send + 'static>(
    player_actions: Receiver<T1>,
    opponent_actions: Receiver<T2>,
) -> Receiver<TetrisPairState> {
    let (tx, rx) = flume::unbounded();
    thread::spawn(move || {
        let mut tetris_pair = TetrisPair::new(10, 20);

        // Setup ganme speed
        let step_delay = time::Duration::from_millis(10);
        tetris_pair.set_fall_speed(1, 30);
        tetris_pair.set_drop_speed(1, 1);
        tetris_pair.set_line_remove_speed(3, 5);

        loop {
            let start = time::Instant::now();
            while let Ok(action) = player_actions.try_recv().map_err(|_|()).and_then(|v| v.try_into()) {
                tetris_pair.add_player_action(PlayerSide::Player, action.0);
            }
            while let Ok(action) = opponent_actions.try_recv().map_err(|_|()).and_then(|v| v.try_into()) {
                tetris_pair.add_player_action(PlayerSide::Opponent, action.0);
            }

            if tetris_pair.step() != (StepResult::None, StepResult::None) {
                tx.send(tetris_pair.get_state()).unwrap();
            }

            let elapsed = start.elapsed();
            if elapsed < step_delay {
                thread::sleep(step_delay - elapsed);
            }
        }
    });
    rx
}

#[allow(dead_code)]
pub fn start_read_key_thread() -> (Receiver<Action>, Receiver<Action>) {
    let term = Term::stdout();
    let (tx_player, rx_player) = flume::unbounded();
    let (tx_opponent, rx_opponent) = flume::unbounded();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        if let Some(action) = key_to_action_player(&key) {
            tx_player.send(action).unwrap();
        }
        if let Some(action) = key_to_action_opponent(&key) {
            tx_opponent.send(action).unwrap();
        }
    });
    (rx_player, rx_opponent)
}

pub fn key_to_action_player(key: &Key) -> Option<Action> {
    match key {
        Key::ArrowLeft => Some(Action::MoveLeft),
        Key::ArrowRight => Some(Action::MoveRight),
        Key::ArrowDown => Some(Action::MoveDown),
        Key::ArrowUp => Some(Action::RotateLeft),
        Key::Char(' ') => Some(Action::Drop),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn key_to_action_opponent(key: &Key) -> Option<Action> {
    match key {
        Key::Char('a') => Some(Action::MoveLeft),
        Key::Char('d') => Some(Action::MoveRight),
        Key::Char('s') => Some(Action::MoveDown),
        Key::Char('w') => Some(Action::RotateLeft),
        Key::Char('q') => Some(Action::Drop),
        _ => None,
    }
}

pub fn render_game_field(term: &Term, state: TetrisPairState, text_player: &[&str], text_opponent: &[&str]) {
    // Cloning every time is highly inoptimal, but dont't care for now
    let field = GameFieldPair::new(state, text_player.iter().map(|s| s.to_string()).collect(), text_opponent.iter().map(|s| s.to_string()).collect());
    let lines = field.render(&AnsiTermStyle);
    term.move_cursor_to(0, 0).unwrap();
    for line in lines {
        term.write_line(&line).unwrap();
    }
}
