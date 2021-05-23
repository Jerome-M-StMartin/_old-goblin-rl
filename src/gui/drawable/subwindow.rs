
//Jerome M. St.Martin
//Node Menu Project
//5/13/2021
/*
use super::Drawable;
use super::super::command::{Command, Commandable};
use super::super::cursor::Cursor;
use super::super::look_n_feel::{ColorOption, Dir};
use super::super::observer::{Observable, Observer};
use super::super::user_input::{InputEvent, UserInput};
use bracket_terminal::prelude::{to_cp437, BTerm, Point};
use std::any::Any;
use std::cell::Cell;
use std::sync::Arc;

pub struct SubWindow {
    pos: Cell<Point>,
    dimensions: (i32, i32), //width, height
    border: bool,
    visible: Cell<bool>,
    ui_subelements: Vec<Arc<dyn Drawable>>,

    cursor: Arc<Cursor>,
    observer_id: usize,
    observable: Arc<dyn Observable>,
}

impl SubWindow {
    pub fn new(pos: Point,
               dimensions: (i32, i32),
               border: bool,
               cursor: Arc<Cursor>,
               observer_id: usize,
               observable: Arc<dyn Observable>) -> Self {
        SubWindow {
            pos: Cell::new(pos),
            dimensions,
            border,
            visible: Cell::new(true),
            ui_subelements: Vec::new(),
            cursor,
            observer_id,
            observable,
        }
    }
    pub fn add_element(&mut self, new_element: Arc<dyn Drawable>) {
        let mut new_pos = self.pos.get();
        new_pos.x += 1;
        new_pos.y += 1;
        new_element.move_to(new_pos);
        self.ui_subelements.push(new_element);
    }
}

impl SubWindow {
    fn close(&self) {
        self.visible.set(false);
    }
}

impl Drawable for SubWindow {
    fn draw(&self, ctx: &mut BTerm) {
        let pos = self.pos.get();
        ctx.draw_box(pos.x, pos.y, self.dimensions.0, self.dimensions.1, ColorOption::DEFAULT.value(), ColorOption::NONE.value());
        for e in self.ui_subelements.iter() {
            e.draw(ctx);
        }
    }
    fn move_to(&self, pos: Point) {
        self.pos.set(pos);
        self.cursor.move_to(pos);
        for e in self.ui_subelements.iter() {
            e.move_to(pos);
        }
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
        self.cursor.orth_move(direction);
        for e in self.ui_subelements.iter() {
            e.orth_move(direction);
        }
    }
}

// --- COMMAND PATTERN ---
impl Commandable<SubWindow> for SubWindow {
    fn send(&self, cmd: Box<dyn Command<SubWindow>>) {
        cmd.execute(self);
    }
}
// --- COMMANDS ---
struct MoveCommand {
    direction: Dir,
}
struct TeleportCommand {
    pos: Point,
}
struct CloseCommand {}

impl Command<SubWindow> for MoveCommand {
    fn execute(&self, subwindow: &SubWindow) {
        subwindow.orth_move(self.direction);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Command<SubWindow> for TeleportCommand {
    fn execute(&self, subwindow: &SubWindow) {
        subwindow.move_to(self.pos);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Command<SubWindow> for CloseCommand {
    fn execute(&self, subwindow: &SubWindow) {
        subwindow.close();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
// --- ---

// --- OBSERVER PATTERN ---
impl Observer for SubWindow {
    fn id(&self) -> usize {
        self.observer_id
    }
    fn update(&self) {
        let input_observable = self.observable.as_any().downcast_ref::<UserInput>();
        if let Some(user_input) = input_observable {
            let input_event = user_input.input.borrow();
            match *input_event {
                Some(InputEvent::WASD(dir)) => {
                    let cmd = MoveCommand {direction: dir};
                    self.send(Box::new(cmd));
                }
                Some(InputEvent::MOUSE(p)) => {
                    let cmd = TeleportCommand { pos: p };
                    self.send(Box::new(cmd));
                }
                Some(InputEvent::ESC) => {
                    let cmd = CloseCommand {};
                    self.send(Box::new(cmd));
                }
                _ => {},
            }
        }
    }
    fn setup_cursor(&self) {
        //Call when this Observer becomes the Focus.
        let pos = self.pos.get();
        let new_pos = Point {
            x: pos.x,
            y: pos.y,
        };
        self.cursor.move_to(new_pos);
        self.cursor.set_glyph(to_cp437('x'));
    }
}
// --- ---
*/
