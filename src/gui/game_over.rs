//Jerome M. St.Martin
//05/24/2021

use std::rc::Rc;
use std::any::Any;
use std::cell::Cell;

use bracket_terminal::prelude::{BTerm, Point};

use super::look_n_feel::{ColorOption, Dir};
use super::drawable::Drawable;
use super::observer::{Observer, Observable};
use super::command::{Command, Commandable};
use super::user_input::{InputEvent, UserInput};

#[derive(PartialEq, Copy, Clone)]
pub enum Selection { NewGame, Quit }

pub struct GameOver {
    pos: Point,
    selection: Cell<Selection>,
    selection_made: Cell<bool>,

    observer_id: usize,
    user_input: Rc<dyn Observable>,
}

impl GameOver {
    pub fn new(user_input: Rc<UserInput>) -> Self {
        GameOver {
            pos: Point {x:0,y:0},
            selection: Cell::new(Selection::NewGame),
            selection_made: Cell::new(false),
            observer_id: user_input.id_gen.generate_observer_id(),
            user_input,
        }
    }
    pub fn get_selection(&self) -> Option<Selection> {
        if self.selection_made.get() {
            return Some(self.selection.get());
        }
        None
    }
    fn change_selection(&self, direction: Dir) {
        match (direction, self.selection.get()) {
            (Dir::UP, Selection::NewGame) => self.selection.set(Selection::Quit),
            (Dir::RIGHT, Selection::NewGame) => self.selection.set(Selection::Quit),

            (Dir::UP, Selection::Quit) => self.selection.set(Selection::NewGame),
            (Dir::RIGHT, Selection::Quit) => self.selection.set(Selection::NewGame),

            (Dir::DOWN, Selection::NewGame) => self.selection.set(Selection::Quit),
            (Dir::LEFT, Selection::NewGame) => self.selection.set(Selection::Quit),

            (Dir::DOWN, Selection::Quit) => self.selection.set(Selection::NewGame),
            (Dir::LEFT, Selection::Quit) => self.selection.set(Selection::NewGame),
        };
    }
}

impl Drawable for GameOver {
    fn draw(&self, ctx: &mut BTerm) {
        //let save_exists = saveload_system::does_save_exist();
        ctx.print_color_centered(15, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "GAME OVER");
        ctx.print_color_centered(24, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "New Game");
        ctx.print_color_centered(25, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "Quit Game");
        match self.selection.get() {
            Selection::NewGame => {
                ctx.print_color_centered(24, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "New Game");
            },
            Selection::Quit => {
                ctx.print_color_centered(25, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "Quit Game");
            }
        }
    }
    fn move_to(&self, _pos: Point) {
        //self.pos = pos;
    }
    fn orth_move(&self, _direction: Dir) {
        /*match direction {
            Dir::UP => self.pos.y -= 1,
            Dir::DOWN => self.pos.y += 1,
            Dir::LEFT => self.pos.x -= 1,
            Dir::RIGHT => self.pos.x += 1,
            _ => {},
        }*/
    }
    fn as_any(&self) -> &dyn Any { self }
}



//===================================
//======== Observer Pattern =========
//===================================
impl Observer for GameOver {
    fn id(&self) -> usize { self.observer_id }
    fn update(&self) {
        let observable = self.user_input.as_any().downcast_ref::<UserInput>();
        if let Some(user_input) = observable {
            if let Some(input_event) = user_input.input.get() {
                match input_event {
                    InputEvent::HJKL(dir) | InputEvent::WASD(dir) => {
                        self.send(Box::new(ChangeSelectionCommand::new(dir)));
                    },
                    InputEvent::ENTER => {
                        self.send(Box::new(SelectCommand::new()));
                    }
                    _ => {},
                }
            }
        }
    }
    fn setup_cursor(&self) {} //this needs to leave this trait
}


//===================================
//======== Command Pattern ==========
//===================================
impl Commandable<GameOver> for GameOver {
    fn send(&self, cmd: Box<dyn Command<GameOver>>) {
        cmd.execute(self);
    }
}

//======= Commands =======
struct ChangeSelectionCommand {
    direction: Dir,
}
impl ChangeSelectionCommand {
    pub fn new(direction: Dir) -> Self {
        ChangeSelectionCommand { direction }
    }
}
impl Command<GameOver> for ChangeSelectionCommand {
    fn execute(&self, main_menu: &GameOver) {
        main_menu.change_selection(self.direction);
    }
    fn as_any(&self) -> &dyn Any { self }
}

struct SelectCommand {}
impl SelectCommand {
    pub fn new() -> Self {
        SelectCommand {}
    }
}
impl Command<GameOver> for SelectCommand {
    fn execute(&self, main_menu: &GameOver) {
        main_menu.selection_made.set(true);
    }
    fn as_any(&self) -> &dyn Any { self }
}
