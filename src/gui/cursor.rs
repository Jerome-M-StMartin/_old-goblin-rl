//Jerome M. St.Martin
//Node Menu Project
//12/07/2020

use super::command::{Command, /*CommandHistory,*/ Commandable};
use super::drawable::Drawable;
use super::look_n_feel::{ColorOption, Dir};
use super::observer::{Observable, Observer};
use super::user_input::{InputEvent, UserInput};
use bracket_terminal::prelude::{to_cp437, BTerm, FontCharType, Point};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

//This struct is shared and should only have one instance, alias as Arc<Cursor>.
pub struct Cursor {
    pub pos: Mutex<Point>,
    pub glyph: Mutex<Option<FontCharType>>,
    pub color: Mutex<ColorOption>,
    pub bg: Mutex<ColorOption>,

    observer_id: usize,
    to_observe: Arc<dyn Observable>,

    //cmd_history: CommandHistory<Cursor>,
}

impl Cursor {
    pub fn new(observer_id: usize, to_observe: Arc<dyn Observable>) -> Cursor {
        Cursor {
            pos: Mutex::new(Point { x: 0, y: 0 }),
            glyph: Mutex::new(Some(to_cp437('>'))),
            color: Mutex::new(ColorOption::DEFAULT),
            bg: Mutex::new(ColorOption::FOCUS),
            observer_id,
            to_observe,
        }
    }

    pub fn set_bg(&self, c: ColorOption) {
        if let Ok(val) = self.bg.get_mut() {
            *val = c;
        } else if Err() {
            panic!("Panic in gui::cursor.set_bg(), a Mutex was poisoned.");
        }
    }

    pub fn set_glyph(&self, new_glyph: FontCharType) {
        if let Ok(val) = self.bg.get_mut() {
            *val = new_glyph;
        } else if Err() {
            panic!("Panic in gui::cursor.set_glyph(), a Mutex was poisoned.");
        }
    }

    //Use to-be-implemented particle/animation system
    //Flash the given ColorOption temporarily
    pub fn blink(_: ColorOption) {}

    pub fn undo(&self) {
        println!("No CmdHistory from which to Undo! [gui::cursor.undo()]");
    }
}

impl Drawable for Cursor {
    fn draw(&self, ctx: &mut BTerm) {
        if let Some(glyph) = self.glyph.get() {
            let p: Point = self.pos.get();
            ctx.set(
                p.x,
                p.y,
                ColorOption::FOCUS.value(),
                ColorOption::NONE.value(),
                glyph,
            );
        }
    }
    fn move_to(&self, pos: Point) {
       self.pos.set(pos); 
    }
    fn orth_move(&self, direction: Dir) {
        let mut pos = self.pos.get();
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
        self.pos.set(pos);
    }
    fn as_any(&self) -> &dyn Any { self }
}

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
        let observable = self.to_observe.as_any().downcast_ref::<UserInput>();
        if let Some(user_input) = observable {
            if let Some(input_event) = user_input.input.get() {
                match input_event {
                    InputEvent::HJKL(dir) => {
                        let cmd = MoveCommand::new(dir);
                        //self.send(Box::new(cmd));
                        self.send(Arc::new(cmd));
                    }
                    _ => {}
                }
            }
        }
    }
    fn setup_cursor(&self) {
        //self.set_glyph(to_cp437('*'));
        self.set_bg(ColorOption::FOCUS);
    }
}

//==================================
//===== Command Pattern Stuff ======
//==================================

impl Commandable<Cursor> for Cursor {
    fn send(&self, cmd: Box<dyn Command<Cursor>>) {
        //What cmd type is this cmd?
        let cmd_type: TypeId = cmd.as_any().type_id();

        //Push cmd to history if possible for this cmd type
        if cmd_type == TypeId::of::<MoveCommand>() {
            let move_cmd = cmd.as_any().downcast_ref::<MoveCommand>();
            let inverse: Dir = opposite_dir(&move_cmd.unwrap().move_direction);
            self.cmd_history.push(MoveCommand::new(inverse));
        }

        cmd.execute(self);
    }
}

//======== COMMANDS ==========
pub struct MoveCommand {
    move_direction: Dir,
}
impl MoveCommand {
    pub fn new(move_direction: Dir) -> Self {
        MoveCommand { move_direction }
    }
}
impl Command<Cursor> for MoveCommand {
    fn execute(&self, cursor: &Cursor) {
        cursor.orth_move(self.move_direction);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct UndoCommand {}
impl UndoCommand {
    pub fn new() -> Self {
        UndoCommand {}
    }
}
impl Command<Cursor> for UndoCommand {
    fn execute(&self, cursor: &Cursor) {
        cursor.undo();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
//===================================

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
