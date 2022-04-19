use std::sync::Arc;

use bracket_lib::prelude::{BTerm, Point};

use super::super::Widget;
use super::super::super::UserInput;

//returns the observer_id of the widget, to allow for needed mutations
//to UserInput, such as making this newly constructed widget the Focus observer.
pub fn construct(ctx: &BTerm, user_input: &Arc<UserInput>) -> usize {
    let (x_chars, y_chars) = ctx.get_char_size();
    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let mut game_over = Widget::new("GameOver",
                Point { x: center_x - 6, y: center_y - 3 },
                Point { x: 11, y: 4 },
                &user_input,
    );
    game_over.with("New Game");
    game_over.with("Quit Game");
    let id: usize = game_over.build();

    return id;
}
