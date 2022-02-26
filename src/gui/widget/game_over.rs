use std::sync::Arc;

use bracket_lib::prelude::Point;

use super::Widget;
use super::super::UserInput;

pub fn construct(user_input: Arc<UserInput>) {
    Widget::new("GameOver",
                Point { x: 14, y: 35 },
                Point { x: 11, y: 5 },
                user_input,
    )
    .with("New Game")
    .with("Quit Game")
    .build();
}
