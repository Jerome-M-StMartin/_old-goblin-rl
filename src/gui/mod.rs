//Jerome M. St.Martin
//jeromemst.martin@gmail.com

#![allow(unused_must_use)]
#![allow(dead_code)]
extern crate rand;
extern crate serde;
use ::std::sync::{Arc, Weak, RwLock};
use bracket_terminal::prelude::BTerm;

mod command;
mod cursor;
mod drawable;
mod look_n_feel;
mod observer;
mod user_input;
mod yaml_parser;
mod main_menu;

use cursor::Cursor;
use drawable::Drawable;
use observer::{Observable, Observer};
use user_input::UserInput;
use std::collections::HashMap;

pub struct GUI {
    user_input: Arc<UserInput>,
    cursor: Arc<RwLock<Cursor>>,

    //NEW CONCEPT:
    //While considering if there is a difference between the observer pattern and
    //some 'signal pattern' I was visualizing, I realized that these are both
    //somewhat poorly named. Instead of thinking of cross-thread communication as
    //the transmission of light, sound, or other physical thing, I now prefer
    //to think of it as a chalkboard being concurrently written on by many
    //polite individuals. No chalk-writer would be so rude as to write where
    //another is currently writting, nor would a reader push a writer out of the
    //way while they are currently writing to see what it is they haven't finished
    //writing yet, nor would any polite & attentive reader or writer become confused
    //or upset if a writer wrote on some part of the chalkboard that no one else was
    //currently reading or writing from.
    //tl;dr: Use shared data with a dirty flag, not back-and-forth data-packets.
    
    //Both parties, ecs and gui, need to handle a single shared instance of this struct.
    drawables: Arc<RwLock<HashMap<usize, Weak<dyn Drawable>>>>, //all ecs Drawable components.
    to_draw: Arc<RwLock<Vec<usize>>>, //these ecs entities need a .draw() call this tick
}

/*-----example Drawable component
#[derive(Component)]
struct GuiData {
    infocard: Arc<InfoCard>,
}
struct InfoCard {}
impl Drawable for InfoCard {}
------------------------------*/


impl GUI {
    //creates, initializes, and returns the gui object
    pub fn new() -> Self {
        // --- Initialize ---
        
        // - UI - 1
        let user_input = Arc::new(UserInput::new());

        // - CURSOR - 
        let cursor_observer_id = user_input.id_gen.generate_observer_id();
        let cursor = Cursor::new(cursor_observer_id, user_input.clone());
        let rwlock_cursor = RwLock::new(cursor);
        let arc_cursor = Arc::new(rwlock_cursor);
        let arc_cursor_as_observer: Arc<RwLock<dyn Observer>> = arc_cursor.clone();

        // - UI - 2
        user_input.add_observer(Arc::downgrade(&arc_cursor_as_observer));
        
        GUI {
            user_input,
            cursor: arc_cursor,
            drawables: Arc::new(RwLock::new(HashMap::new())),
            to_draw: Arc::new(RwLock::new(Vec::new())),
        }
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        let input_is_dirty: bool = self.user_input.transcribe_input(ctx); //read & translate input event from BTerm
        if input_is_dirty {
            self.user_input.notify_focus(); //notify only the active observer (i.e. the focus)
        }

        //draw all to_draw entities from drawables
        if let Ok(id_vec) = self.to_draw.read() {
            if !id_vec.is_empty() {
                if let Ok(drawables) = self.drawables.read() {
                    for entity_id in id_vec.iter() {
                        if let Some(weak_drawable) = drawables.get(entity_id) {
                            if let Some(drawable) = weak_drawable.upgrade() {
                                drawable.draw(ctx);
                            }
                        }
                    }
                }
            }
        }
        if let Ok(c) = self.cursor.read() {
            c.draw(ctx); //draw the cursor
        }
    }

    pub fn add_drawable(&self, entity_id: usize, new_drawable: &Arc<dyn Drawable>) {
        let to_add = Arc::downgrade(new_drawable);
        if let Ok(mut drawables) = self.drawables.write() {
            drawables.insert(entity_id, to_add);
        }
    }

    //pub fn add_hud_data(&self, new_hud_data: HudData) {}

    pub fn draw_hud(&self, ctx: &mut BTerm, show_tooltips: bool) {}
}
