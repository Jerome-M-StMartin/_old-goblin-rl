//Jerome M. St.Martin
//Node Menu Project
//12/23/2020

//The Idea:
//Self-contained ScrollBar object that accepts Commands when
//it is the Focus. Shows a movable window into a Vec<&T>, defined
//by the slice_bounds field.

use super::Drawable;
use super::super::command::{Command, CommandHistory, Commandable};
use super::super::cursor::Cursor;
use super::super::look_n_feel::{ColorOption, Dir};
use super::super::observer::{Observable, Observer};
use super::super::user_input::{InputEvent, UserInput};
use bracket_terminal::prelude::{to_cp437, BTerm, FontCharType, Point};
use std::any::Any;
use std::cell::Cell;
use std::fmt::Display;
use std::sync::Arc;

pub struct ScrollBar<T> {
    pos: Cell<Point>,
    slice_bounds: Cell<(usize, usize)>, //portion of content vec to display
    content: Vec<T>,                       //<-Should be a Cell/RefCell<> in the future, probably
    cursor: Arc<Cursor>,

    //Observer Pattern Stuff
    observer_id: usize,
    observable: Arc<dyn Observable>,

    //Command Pattern Stuff
    cmd_hist: CommandHistory<ScrollBar<T>>,
}

impl<T> ScrollBar<T> {
    pub fn new(
        pos: Point,
        content: Vec<T>,
        depth: u8,
        cursor: Arc<Cursor>,
        observer_id: usize,
        observable: Arc<dyn Observable>,
    ) -> Self {
        let sb = ScrollBar {
            pos: Cell::new(pos),
            slice_bounds: Cell::new((0, depth as usize)),
            content,
            cursor,
            observer_id,
            observable,
            cmd_hist: CommandHistory::new(),
        };
        return sb;
    }
    fn scroll(&self, direction: Dir) {
        let mut slice_bounds = self.slice_bounds.get();
        if direction == Dir::UP && slice_bounds.0 > 0 {
            slice_bounds.0 -= 1;
            slice_bounds.1 -= 1;
        } else if direction == Dir::DOWN && slice_bounds.1 < self.content.len() {
            slice_bounds.0 += 1;
            slice_bounds.1 += 1;
        }
        self.slice_bounds.set(slice_bounds);
    }
    fn move_cursor(&self, direction: Dir) {
        let pos = self.pos.get();
        let cursor = self.cursor.clone();
        let slice_bounds = self.slice_bounds.get();
        match direction {
            Dir::UP => {
                if cursor.pos.get().y == pos.y {
                    self.scroll(direction);
                } else {
                    cursor.orth_move(direction);
                }
            }
            Dir::DOWN => {
                if cursor.pos.get().y == pos.y + (slice_bounds.1 as i32 - slice_bounds.0 as i32 - 1) {
                    self.scroll(direction);
                } else {
                    cursor.orth_move(direction);
                }
            }
            _ => {}
        }
    }
}

impl<T: Display + 'static> Drawable for ScrollBar<T> {
    fn draw(&self, ctx: &mut BTerm) {
        let slice_bounds = self.slice_bounds.get();
        let pos = self.pos.get();

        let start = slice_bounds.0;
        let end = slice_bounds.1;
        let len = self.content.len();
        let mut glyph: FontCharType;
        for idx in start..end {
            if idx == 0 {
                glyph = to_cp437('┌');
            } else if idx == start {
                glyph = to_cp437('↑');
            } else if idx == (len - 1) {
                glyph = to_cp437('└');
            } else if idx == (end - 1) {
                glyph = to_cp437('↓');
            } else {
                glyph = to_cp437('├');
            }
            ctx.set(
                pos.x,
                pos.y + (idx - start) as i32,
                ColorOption::DEFAULT.value(),
                ColorOption::NONE.value(),
                glyph,
            );
            ctx.set(
                pos.x + 1,
                pos.y + (idx - start) as i32,
                ColorOption::DEFAULT.value(),
                ColorOption::NONE.value(),
                to_cp437('•'),
            );
            ctx.print(
                pos.x + 2,
                pos.y + (idx - start) as i32,
                self.content[idx].to_string(),
            )
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
}

//==================================
//===== Observer Pattern Stuff =====
//==================================

impl<T: 'static> Observer for ScrollBar<T> {
    fn id(&self) -> usize {
        self.observer_id
    }
    fn update(&self) {
        let input_observable = self.observable.as_any().downcast_ref::<UserInput>();
        if let Some(user_input) = input_observable {
            let input_event = user_input.input.borrow();
            match *input_event {
                Some(InputEvent::WASD(dir)) => {
                    //let cmd = ScrollCommand::new(dir);
                    let cmd = MoveCursorCommand::new(dir);
                    self.send(Box::new(cmd));
                }
                _ => {}
            }
        }
    }
    fn setup_cursor(&self) {
        //Call this whenever this Observer becomes the Focus.
        let new_pos = Point {
            x: self.pos.get().x + 1,
            y: self.pos.get().y,
        };
        self.cursor.move_to(new_pos);
        self.cursor.set_glyph(to_cp437('○'));
    }
}

//==================================
//===== Command Pattern Stuff ======
//==================================

impl<T> Commandable<ScrollBar<T>> for ScrollBar<T> {
    fn send(&self, cmd: Box<dyn Command<ScrollBar<T>>>) {
        cmd.execute(self);
    }
}

//======== COMMANDS ==========
pub struct ScrollCommand {
    direction: Dir, //Up or Down, in this case
}
impl ScrollCommand {
    pub fn new(direction: Dir) -> Self {
        ScrollCommand { direction }
    }
}
impl<T> Command<ScrollBar<T>> for ScrollCommand {
    fn execute(&self, scrollbar: &ScrollBar<T>) {
        scrollbar.scroll(self.direction);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct MoveCursorCommand {
    direction: Dir,
}
impl MoveCursorCommand {
    pub fn new(direction: Dir) -> Self {
        MoveCursorCommand { direction }
    }
}
impl<T> Command<ScrollBar<T>> for MoveCursorCommand {
    fn execute(&self, scrollbar: &ScrollBar<T>) {
        scrollbar.move_cursor(self.direction);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
//============================

/*
 * ┌•
 * ├○ <--selected
 * ├•
 * ↓
 *
 * ↑
 * ├•
 * ├•
 * ├•
 * ↓
 *
 * ↑
 * ├•
 * ├•
 * └•
 */
