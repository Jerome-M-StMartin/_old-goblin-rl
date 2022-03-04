//Jerome M. St.Martin
//Node Menu Project
//12/07/2020

use super::super::{RunState, World};

use super::command::{Command, Commandable, CommandQueue};
use super::look_n_feel::{ColorOption, Dir};
use super::observer::Observer;
use super::user_input::{InputEvent, UserInput};
use bracket_lib::prelude::{to_cp437, FontCharType, Point};
use std::sync::{Arc, Mutex};

//This struct is shared and should only have one instance, alias as Arc<Cursor>.
pub struct Cursor {
    name: String,
    pub pos: Mutex<Point>,
    pub glyph: Mutex<Option<FontCharType>>,
    pub color: Mutex<ColorOption>,
    pub bg: Mutex<ColorOption>,

    user_input: Arc<UserInput>,

    observer_id: usize,
    to_observe: Arc<UserInput>,
    //to_observe: Arc<dyn Observable>,

    cmd_queue: CommandQueue,
}

impl Cursor {
    pub fn new(user_input: &Arc<UserInput>, observer_id: usize, to_observe: Arc<UserInput>) -> Cursor {
        Cursor {
            name: "gui::Cursor".to_string(),
            pos: Mutex::new(Point { x: 0, y: 0 }),
            glyph: Mutex::new(Some(to_cp437('>'))),
            color: Mutex::new(ColorOption::DEFAULT),
            bg: Mutex::new(ColorOption::FOCUS),
            user_input: user_input.clone(),
            observer_id,
            to_observe,
            cmd_queue: CommandQueue::new(),
        }
    }

    pub fn orth_move(&self, dir: Dir) {
        if let Ok(mut pos) = self.pos.lock() {
            match dir {
                Dir::UP => { pos.y -= 1 },
                Dir::DOWN => { pos.y += 1 },
                Dir::LEFT => { pos.x -= 1 },
                Dir::RIGHT => { pos.x += 1 },
            }
        }
    }

    pub fn set_bg(&self, c: ColorOption) {
        if let Ok(mut guard) = self.bg.lock() {
            *guard = c;
        } else { panic!("Panic in gui::cursor.set_bg(), a Mutex was poisoned."); }
    }

    pub fn set_glyph(&self, new_glyph: FontCharType) {
        if let Ok(mut guard) = self.glyph.lock() {
            *guard = Some(new_glyph);
        } else { panic!("Panic in gui::cursor.set_glyph(), a Mutex was poisoned."); }
    }

    //Use to-be-implemented particle/animation system
    //Flash the given ColorOption temporarily
    pub fn blink(_: ColorOption) {}

    pub fn undo(&self) {
        println!("No CmdHistory from which to Undo! [gui::cursor.undo()]");
    }
}

/*impl Drawable for Cursor {
    fn draw(&self, ctx: &mut BTerm) {
        if let Ok(guard) = self.glyph.lock() {
            if let Some(glyph) = *guard {
                if let Ok(p) = self.pos.lock() {
                    ctx.set(
                        p.x,
                        p.y,
                        ColorOption::FOCUS.value(),
                        ColorOption::NONE.value(),
                        glyph,
                    );
                }
            }
        }
    }
    fn move_to(&self, pos: Point) {
        if let Ok(mut guard) = self.pos.lock() {
            *guard = pos;
        }
    }
    fn orth_move(&self, direction: Dir) {
        println!("cursor.orth_move({:?})", &direction);
        if let Ok(mut pos) = self.pos.lock() {

            match direction {
                Dir::UP => { 
                    pos.y -= 1;
                },
                Dir::DOWN => { 
                    pos.y += 1;
                },
                Dir::LEFT => { 
                    pos.x -= 1;
                },
                Dir::RIGHT => { 
                    pos.x += 1;
                },
            }
        }
    }
    fn as_any(&self) -> &dyn Any { self }
}*/

//==================================
//==== Observer Pattern Stuff ======
//==================================
//Only for when the Cursor is itself the Focus observer of the UserInput Observable.
//Else, the current Focus will control the Cursor through its shared reference.
impl Observer for Cursor {
    fn id(&self) -> usize {
        self.observer_id
    }
    fn update(&self) {
        if let Ok(input_event_guard) = self.user_input.input.read() {
            if let Some(input_event) = *input_event_guard {
                let cmd_option: Option<Command> = match input_event {
                    InputEvent::HJKL(dir) => { Some(Command::Move{dir}) },
                    _ => { None },
                };
                if let Some(cmd) = cmd_option {
                    self.send(cmd);
                }
            }
        }
    }
    fn become_focus(&self) {
        self.set_bg(ColorOption::FOCUS);
    }
    fn name(&self) -> &str {
        &self.name
    }
}

//==================================
//===== Command Pattern Stuff ======
//==================================

impl Commandable for Cursor {
    fn send(&self, cmd: Command) {
        self.cmd_queue.push(cmd)
    }

    fn gui_process(&self) {
        for cmd in &self.cmd_queue.iter() {
            match cmd {
                Command::Move{dir} => { self.orth_move(*dir); },
                _ => {},
            }
        }

        self.cmd_queue.clear();
    }

    fn ecs_process(&self, _ecs: &mut World, runstate: RunState) -> RunState {
        //donothing
        runstate
    }
}

//local helper
fn opposite_dir(d: &Dir) -> Dir {
    match d {
        Dir::UP => return Dir::DOWN,
        Dir::DOWN => return Dir::UP,
        Dir::LEFT => return Dir::RIGHT,
        Dir::RIGHT => return Dir::LEFT,
    }
}

/*This is the pre-threadsafe implementation.
 *
 * pub struct Cursor {
    pub pos: Cell<Point>,                  // position
    pub glyph: Cell<Option<FontCharType>>, // char that visually represents cursor
    pub color: Cell<ColorOption>,          // glyph color
    pub bg: Cell<ColorOption>,             // background color

    //Observer Pattern Fields
    observer_id: usize,
    to_observe: Rc<dyn Observable>,

    //Command Pattern Field
    cmd_history: CommandHistory<Cursor>, // stateful expansion to Command Pattern
}

impl Cursor {
    pub fn new(observer_id: usize, to_observe: Rc<dyn Observable>) -> Cursor {
        Cursor {
            pos: Cell::new(Point { x: 0, y: 0 }),
            glyph: Cell::new(Some(to_cp437('>'))),
            color: Cell::new(ColorOption::DEFAULT),
            bg: Cell::new(ColorOption::FOCUS),
            observer_id,
            to_observe,
            cmd_history: CommandHistory::new(),
        }
    }

    pub fn set_bg(&self, c: ColorOption) {
        self.bg.set(c);
    }

    pub fn set_glyph(&self, new_glyph: FontCharType) {
        self.glyph.set(Some(new_glyph));
    }

    //Use to-be-implemented particle/animation system
    //Flash the given ColorOption temporarily
    pub fn blink(_: ColorOption) {}

    pub fn undo(&self) {
        match self.cmd_history.pop() {
            Ok(cmd) => {
                cmd.execute(self);
            }
            Err(e) => {
                eprintln!("Attempted cmd_history.pop() but:\n  {}", e);
            }
        }
    }
}*/
