use std::sync::Arc;

use bracket_lib::prelude::Point;

use super::Widget;
use super::super::UserInput;

pub fn construct(user_input: &Arc<UserInput>) {
    let mut main_menu = Widget::new("MainMenu",
                Point { x: 14, y: 1 },
                Point { x: 11, y: 5 },
                &user_input,
    );
    main_menu.with("New Game");
    main_menu.with("Load Game");
    main_menu.with("Quit Game");
    main_menu.build();
}
