//Jerome M. St.Martin
//jeromemst.martin@gmail.com

#![allow(unused_must_use)]
#![allow(dead_code)]
extern crate rand;
extern crate serde;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use bracket_terminal::prelude::BTerm;

mod command;
mod yaml_parser;

pub mod textify;
pub mod look_n_feel;
pub mod observer;
pub mod main_menu;
pub mod game_over;
pub mod cursor;
pub mod drawable;

use cursor::Cursor;
use drawable::Drawable;
use observer::Observable;
use super::user_input::UserInput;
use super::user_input;

pub use observer::Observer;
pub use main_menu::MainMenu;
pub use game_over::GameOver;

pub struct GUI {
    pub user_input: Rc<UserInput>,
    pub cursor: Rc<Cursor>,

    //DO THESE NEED TO BE Rc<> ??? I don't think so anymore, please verify.------------!
    drawables: RefCell<HashMap<usize, Weak<dyn Drawable>>>, //all Drawable components.
    to_draw: RefCell<Vec<usize>>, //these ecs entities need a .draw() call this tick
}

/*-----example Drawable ecs component
#[derive(Component)]
struct GuiData {
    infocard: Arc<InfoCard>,
}
struct InfoCard {}
impl Drawable for InfoCard {}
------------------------------*/

impl GUI {
    //creates, initializes, and returns the gui object
    pub fn new(user_input: Rc<UserInput>) -> Self {
        // --- Initialize ---
        let drawables: HashMap<usize, Weak<dyn Drawable>> = HashMap::new();
        let to_draw: Vec<usize> = Vec::new();
        
        // - CURSOR - 
        let cursor_observer_id = user_input.id_gen.generate_observer_id();
        let cursor = Cursor::new(cursor_observer_id, user_input.clone());
        let rc_cursor = Rc::new(cursor);
        let rc_cursor_as_observer: Rc<dyn Observer> = rc_cursor.clone();
        user_input.add_observer(Rc::downgrade(&rc_cursor_as_observer));
        
        GUI {
            user_input,
            cursor: rc_cursor,
            drawables: RefCell::new(drawables),
            to_draw: RefCell::new(to_draw),
        }
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        let input_is_dirty: bool = self.user_input.transcribe_input(ctx); //read & translate input event from BTerm
        if input_is_dirty {
            self.user_input.notify_focus(); //notify only the active observer (i.e. the focus)
        }

        //draw all to_draw gui objs & remove dropped references
        let mut drawable_drops: Vec<usize> = Vec::new();
        let mut to_draw_drops: Vec<usize> = Vec::new();
        let mut idx = 0;
        let mut to_draw = self.to_draw.borrow_mut();
        for id in to_draw.iter() {
            if let Some(weak_drawable) = self.drawables.borrow().get(id) {
                if let Some(drawable) = weak_drawable.upgrade() {
                    drawable.draw(ctx);
                } else { 
                    drawable_drops.push(*id);
                    to_draw_drops.push(idx);
                }
            }
            idx += 1;
        }

        self.cursor.draw(ctx); //draw the cursor

        //lazy removal of dropped references
        for id in drawable_drops.iter() {
            self.drawables.borrow_mut().remove(id);
        }
        for idx in to_draw_drops.iter() {
            to_draw.remove(*idx);
        }
    }

    pub fn add_drawable(&self, id: usize, to_add: Rc<dyn Drawable>, set_focus: bool) {
        self.drawables.borrow_mut().insert(id, Rc::downgrade(&to_add));
        self.to_draw.borrow_mut().push(id);

        if set_focus {
            self.user_input.set_focus(id);
        }
    }

    pub fn rm_drawable(&self, id: usize) {
        self.drawables.borrow_mut().remove(&id);
        //id of this drawable is removed from to_draw lazily, also this drawable itself will be
        //lazily removed if and only if all non-Weak Rc references are dropped. Thus the purpose of
        //this function: to remove a drawable that must retain a strong Rc reference somewhere.
    }

    //pub fn add_hud_data(&self, new_hud_data: HudData) {}
}
