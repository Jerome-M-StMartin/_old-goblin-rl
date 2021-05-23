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
mod look_n_feel;
mod observer;
mod yaml_parser;
mod main_menu;

pub mod cursor;
pub mod drawable;
pub mod user_input;

use cursor::Cursor;
use drawable::Drawable;
use observer::{Observable, Observer};
use user_input::UserInput;

pub use main_menu::MainMenu;

pub struct GUI {
    pub user_input: Rc<UserInput>,
    pub cursor: Rc<Cursor>,

    //Both parties, ecs and gui, need to handle a single shared instance of these:
    drawables: Rc<RefCell<HashMap<usize, Weak<dyn Drawable>>>>, //all ecs Drawable components.
    to_draw: Rc<RefCell<Vec<usize>>>, //these ecs entities need a .draw() call this tick
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
    pub fn new(user_input: Rc<UserInput>, main_menu: Rc<MainMenu>) -> Self {
        // --- Initialize ---
        let mut drawables: HashMap<usize, Weak<dyn Drawable>> = HashMap::new();
        let mut to_draw: Vec<usize> = Vec::new();
        
        // - CURSOR - 
        let cursor_observer_id = user_input.id_gen.generate_observer_id();
        let cursor = Cursor::new(cursor_observer_id, user_input.clone());
        let rc_cursor = Rc::new(cursor);

        // - MAIN MENU -
        let mm_weak = Rc::downgrade(&main_menu);
        drawables.insert(main_menu.id(), mm_weak);
        to_draw.push(main_menu.id());

        // - UI - 2
        let rc_cursor_as_observer: Rc<dyn Observer> = rc_cursor.clone();
        user_input.add_observer(Rc::downgrade(&rc_cursor_as_observer));
        let main_menu_as_observer: Rc<dyn Observer> = main_menu.clone();
        user_input.add_observer(Rc::downgrade(&main_menu_as_observer));
        
        GUI {
            user_input,
            cursor: rc_cursor,
            drawables: Rc::new(RefCell::new(drawables)),
            to_draw: Rc::new(RefCell::new(to_draw)),
        }
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        let input_is_dirty: bool = self.user_input.transcribe_input(ctx); //read & translate input event from BTerm
        if input_is_dirty {
            self.user_input.notify_focus(); //notify only the active observer (i.e. the focus)
        }

        //draw all to_draw entities from drawables
        for entity_id in self.to_draw.borrow_mut().iter() {
            if let Some(weak_drawable) = self.drawables.borrow().get(entity_id) {
                if let Some(drawable) = weak_drawable.upgrade() {
                    drawable.draw(ctx);
                }
            }
        }

        self.cursor.draw(ctx); //draw the cursor
    }

    pub fn add_drawable(&self, entity_id: usize, new_drawable: &Rc<dyn Drawable>) {
        let to_add = Rc::downgrade(new_drawable);
        self.drawables.borrow_mut().insert(entity_id, to_add);
    }

    //pub fn add_hud_data(&self, new_hud_data: HudData) {}
}
