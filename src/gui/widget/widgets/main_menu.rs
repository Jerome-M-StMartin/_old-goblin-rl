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

    let mut main_menu = Widget::new("MainMenu",
                Point { x: center_x -  6, y: center_y - 3 },
                Point { x: 11, y: 5 },
                &user_input,
    );
    main_menu.with("New Game");
    main_menu.with("Load Game");
    main_menu.with("Quit Game");
    let id: usize = main_menu.build();

    return id;
}
