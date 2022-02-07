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
pub mod main_menu;
pub mod game_over;
pub mod cursor;
//pub mod drawable;
pub mod widget;
pub mod gamelog;

use cursor::Cursor;
//use drawable::Drawable;
use super::user_input::UserInput;
use super::user_input;
use super::command;

pub use observer::Observable;
pub use observer::Observer;
pub use main_menu::MainMenu;
pub use game_over::GameOver;

pub struct GUI {
    pub user_input: Arc<UserInput>,
    pub cursor: Arc<Cursor>,

    //drawables: RefCell<HashMap<usize, Weak<dyn Drawable>>>, //all Drawable components.
    //to_draw: RefCell<Vec<usize>>, //these ecs entities need a .draw() call this tick
}

impl GUI {
    //creates, initializes, and returns the gui object
    pub fn new(user_input: Arc<UserInput>) -> Self {
        // --- Initialize ---
        let gui: Self;
        let arc_cursor: Arc<Cursor>;
        //let drawables: HashMap<usize, Weak<dyn Drawable>> = HashMap::new();
        //let to_draw: Vec<usize> = Vec::new();
        
        // - CURSOR - 
        if let Ok(guard) = user_input.id_gen.lock() {
            let cursor_observer_id = guard.generate_observer_id();
            let cursor = Cursor::new(user_input.clone(), cursor_observer_id, user_input.clone());
            arc_cursor = Arc::new(cursor);
            let arc_cursor_as_observer: Arc<dyn Observer> = arc_cursor.clone();
            user_input.add_observer(Arc::downgrade(&arc_cursor_as_observer));
        
            gui = GUI {
                user_input: user_input.clone(),
                cursor: arc_cursor,
                //drawables: RefCell::new(drawables),
                //to_draw: RefCell::new(to_draw),
            };
        } else { panic!("Mutex on user_input.id_gen was poisoned.") };

        gui
    }

    //call this function in the main bracket-lib game loop.
    pub fn tick(&mut self, ctx: &mut BTerm) {
        self.user_input.tick(ctx);

        /*//draw all to_draw gui objs & remove dropped references
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

        //lazy removal of dropped references
        for id in drawable_drops.iter() {
            self.drawables.borrow_mut().remove(id);
        }
        for idx in to_draw_drops.iter() {
            to_draw.remove(*idx);
        }*/

        self.cursor.draw(ctx); //draw the cursor
    }

    //Not to be called in self::tick(), so as to allow finer control.
    // HUD state is updated in tick(), but we need to ensure that we draw it at the end of the frame
    // so it properly overlays all other rendered things.
    pub fn draw_hud(&self) {
        
    }
     /*
    pub fn add_drawable(&self, id: usize, to_add: Arc<dyn Drawable>, set_focus: bool) {
        self.drawables.borrow_mut().insert(id, Arc::downgrade(&to_add));
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
    }*/
}
