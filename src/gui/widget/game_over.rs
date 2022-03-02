use std::sync::Arc;

use bracket_lib::prelude::Point;

use super::Widget;
use super::super::UserInput;

pub fn construct(user_input: &Arc<UserInput>) {
    let mut game_over = Widget::new("GameOver",
                Point { x: 14, y: 35 },
                Point { x: 11, y: 5 },
                &user_input,
    );
    game_over.with("New Game");
    game_over.with("Quit Game");
    game_over.build();
}
