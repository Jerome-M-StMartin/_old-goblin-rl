//Jerome M. St.Martin
//jeromemst.martin@gmail.com

#![allow(unused_must_use)]
#![allow(dead_code)]
extern crate rand;
extern crate serde;
use std::sync::Arc;
use bracket_lib::prelude::{BTerm, DrawBatch, render_draw_buffer};

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

    pub fn init_widgets(&self) {
        widget::widgets::player_stats::construct(&self.user_input);
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        self.user_input.tick(ctx);
        
        widget::widget_storage::update_all();
        //println!("{:?}", widget::WIDGET_DATA.lock().unwrap().get("PlayerStats").unwrap()[0]);

        let mut draw_batch = DrawBatch::new(); //pass mutable borrows of this to each Widget.draw() call.
        widget::widget_storage::draw_all(ctx, &mut draw_batch);
        render_draw_buffer(ctx).expect("Render error in GUI::tick()");
    }
}
