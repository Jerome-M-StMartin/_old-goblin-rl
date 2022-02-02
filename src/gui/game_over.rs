//Jerome M. St.Martin
//05/24/2021

use std::sync::{Arc, Mutex};
use std::any::Any;

use bracket_lib::prelude::{BTerm, Point};

use super::look_n_feel::{ColorOption, Dir};
use super::drawable::Drawable;
use super::observer::{Observer, Observable};
use super::command::{Command, Commandable, CommandQueue};
use super::user_input::{InputEvent, UserInput};

#[derive(PartialEq, Copy, Clone)]
pub enum Selection { NewGame, Quit }

pub struct GameOver {
    name: String,
    pos: Point,
    selection: Mutex<Selection>,
    selection_made: Mutex<bool>,

    observer_id: usize,
    user_input: Arc<UserInput>,

    cmd_queue: CommandQueue,
}

impl GameOver {
    pub fn new(user_input: Arc<UserInput>) -> Self {
        if let Ok(guard) = user_input.id_gen.lock() {
            GameOver {
                name: "GameOver".to_string(),
                pos: Point {x:0,y:0},
                selection: Mutex::new(Selection::NewGame),
                selection_made: Mutex::new(false),
                observer_id: guard.generate_observer_id(),
                user_input: user_input.clone(),
                cmd_queue: CommandQueue::new(),
            }
        } else {
            panic!("Found Mutex was Poinoned in gui::GameOver::new().")
        }
    }
    pub fn get_selection(&self) -> Option<Selection> {
        if let Ok(_) = self.selection_made.lock() {
            if let Ok(guard) = self.selection.lock() {
               return  Some(*guard);
            };
        }
        None
    }
    fn change_selection(&self, direction: Dir) {
        if let Ok(mut curr_selection) = self.selection.lock() {
            let new_selection: Selection = match (direction, *curr_selection) {
                (Dir::UP, Selection::NewGame) => Selection::Quit,
                (Dir::RIGHT, Selection::NewGame) => Selection::Quit,

                (Dir::UP, Selection::Quit) => Selection::NewGame,
                (Dir::RIGHT, Selection::Quit) => Selection::NewGame,

                (Dir::DOWN, Selection::NewGame) => Selection::Quit,
                (Dir::LEFT, Selection::NewGame) => Selection::Quit,

                (Dir::DOWN, Selection::Quit) => Selection::NewGame,
                (Dir::LEFT, Selection::Quit) => Selection::NewGame,
            };

            *curr_selection = new_selection;
        }
    }
}

impl Drawable for GameOver {
    fn draw(&self, ctx: &mut BTerm) {
        //let save_exists = saveload_system::does_save_exist();
        ctx.print_color_centered(15, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "GAME OVER");
        ctx.print_color_centered(24, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "New Game");
        ctx.print_color_centered(25, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "Quit Game");
        if let Ok(guard) = self.selection.lock() {
            match *guard {
                Selection::NewGame => {
                    ctx.print_color_centered(24, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "New Game");
                },
                Selection::Quit => {
                    ctx.print_color_centered(25, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "Quit Game");
                }
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
        if let Ok(input_event_guard) = self.user_input.input.read() {
            if let Some(input_event) = *input_event_guard {
                let cmd_option = match input_event {
                    InputEvent::WASD(dir) => { Some(Command::Move {dir}) },
                    InputEvent::ENTER => { Some(Command::Select) },
                    _ => { None },
                };

                if let Some(cmd) = cmd_option {
                    self.send(cmd);
                }
            }
        }
    }
    fn setup_cursor(&self) {} //this needs to leave this trait
    fn name(&self) -> &str { &self.name }
}


//===================================
//======== Command Pattern ==========
//===================================
impl Commandable for GameOver {
    fn send(&self, cmd: Command) {
        self.cmd_queue.push(cmd);
    }
    fn process(&self, _ecs: &mut super::super::World) -> super::super::RunState {
        let mut next: Option<Command> = self.cmd_queue.pop_front();
        while next.is_some() {
            match next.unwrap() {
                Command::Select => {
                    if let Ok(mut selection_made) = self.selection_made.lock() {
                        *selection_made = true;
                    }
                },
                Command::Move{dir} => {
                    self.change_selection(dir)
                },
                _ => {},
            }
            next = self.cmd_queue.pop_front();
        }
        super::super::RunState::GameOver
    }
}
