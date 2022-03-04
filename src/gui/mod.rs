//Jerome M. St.Martin
//jeromemst.martin@gmail.com

#![allow(unused_must_use)]
#![allow(dead_code)]
extern crate rand;
extern crate serde;
use std::sync::Arc;
use bracket_lib::prelude::BTerm;

pub mod textify;
pub mod look_n_feel;
pub mod observer;
pub mod cursor;
pub mod widget;
pub mod gamelog;

use cursor::Cursor;
//use drawable::Drawable;
use super::user_input::UserInput;
use super::user_input;
use super::command;

pub use observer::Observable;
pub use observer::Observer;

pub struct GUI {
    pub user_input: Arc<UserInput>,
    pub cursor: Arc<Cursor>,
}

impl GUI {
    //creates, initializes, and returns the gui object
    pub fn new(user_input: &Arc<UserInput>) -> Self {
        // --- Initialize ---
        let gui: Self;
        let arc_cursor: Arc<Cursor>;
        
        // - CURSOR - 
        if let Ok(guard) = user_input.id_gen.lock() {
            let cursor_observer_id = guard.generate_observer_id();
            let cursor = Cursor::new(&user_input, cursor_observer_id, user_input.clone());
            arc_cursor = Arc::new(cursor);
            user_input.add_observer(&arc_cursor);
        
            gui = GUI {
                user_input: user_input.clone(),
                cursor: arc_cursor,
            };
        } else { panic!("Mutex on user_input.id_gen was poisoned.") };

        gui
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        self.user_input.tick(ctx);
        widget::widget_storage::draw_all(ctx);
    }

    //Not to be called in self::tick(), so as to allow finer control.
    // HUD state is updated in tick(), but we need to ensure that we draw it at the end of the frame
    // so it properly overlays all other rendered things.
    pub fn draw_hud(&self) {
        
    }
}
