//Jerome M. St.Martin
//05/19/2021

use std::sync::{Arc, Mutex};
use std::any::Any;

use bracket_terminal::prelude::{BTerm, Point};

use super::look_n_feel::{ColorOption, Dir};
use super::drawable::Drawable;
use super::observer::{Observer, Observable};
use super::command::{Command, Commandable, CommandQueue};
use super::user_input::{InputEvent, UserInput};

#[derive(PartialEq, Copy, Clone)]
pub enum Selection { NewGame, LoadGame, Quit }

pub struct MainMenu {
    pos: Point,
    selection: Mutex<Selection>,
    selection_made: Mutex<bool>,

    observer_id: usize,
    user_input: Arc<UserInput>,

    cmd_queue: CommandQueue,
}

impl MainMenu {
    pub fn new(user_input: Arc<UserInput>) -> Self {
        let new_id: usize;
        if let Ok(guard) = user_input.id_gen.lock() {
            new_id = guard.generate_observer_id();
        } else {
            panic!("Mutex poisoned, GUI::MainMenu::new()");
        }

        MainMenu {
            pos: Point {x:0,y:0},
            selection: Mutex::new(Selection::NewGame),
            selection_made: Mutex::new(false),
            observer_id: new_id,
            user_input,
            cmd_queue: CommandQueue::new(),
        }
    }

    pub fn get_selection(&self) -> Option<Selection> {
        if let Ok(selection_made) = self.selection_made.lock() {
            if *selection_made {
                if let Ok(guard) = self.selection.lock() {
                    return Some(*guard);
                };
            } else { return None; };
        }
        panic!("Mutex poisoned, noticed in MainMenu::get_selection().");
    }
    
    fn change_selection(&self, direction: Dir) {
        let variant;
        if let Ok(mut guard) = self.selection.lock() {

            match (direction, *guard) {
                (Dir::UP, Selection::NewGame) => variant = Selection::Quit,
                (Dir::RIGHT, Selection::NewGame) => variant = Selection::Quit,

                (Dir::UP, Selection::LoadGame) => variant = Selection::NewGame,
                (Dir::RIGHT, Selection::LoadGame) => variant = Selection::NewGame,

                (Dir::UP, Selection::Quit) => variant = Selection::LoadGame,
                (Dir::RIGHT, Selection::Quit) => variant = Selection::LoadGame,

                (Dir::DOWN, Selection::NewGame) => variant = Selection::LoadGame,
                (Dir::LEFT, Selection::NewGame) => variant = Selection::LoadGame,

                (Dir::DOWN, Selection::LoadGame) => variant = Selection::Quit,
                (Dir::LEFT, Selection::LoadGame) => variant = Selection::Quit,

                (Dir::DOWN, Selection::Quit) => variant = Selection::NewGame,
                (Dir::LEFT, Selection::Quit) => variant = Selection::NewGame,
            };

            *guard = variant;
        } else {
            panic!("Mutex poisoned, noticed in MainMenu::change_selection().")
        }
    }
}

impl Drawable for MainMenu {
    fn draw(&self, ctx: &mut BTerm) {
        //let save_exists = saveload_system::does_save_exist();
        ctx.print_color_centered(15, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "GoblinRL");
        ctx.print_color_centered(24, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "New Game");
        ctx.print_color_centered(25, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "Load Game");
        ctx.print_color_centered(26, ColorOption::DEFAULT.value(), ColorOption::NONE.value(), "Quit Game");

        if let Ok(guard) = self.selection.lock() {
            match *guard {
                Selection::NewGame => {
                    ctx.print_color_centered(24, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "New Game");
                },
                Selection::LoadGame => {
                    ctx.print_color_centered(25, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "Load Game");
                },
                Selection::Quit => {
                    ctx.print_color_centered(26, ColorOption::FOCUS.value(), ColorOption::NONE.value(), "Quit Game");
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
impl Observer for MainMenu {
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
}


//===================================
//======== Command Pattern ==========
//===================================
impl Commandable for MainMenu {
    fn send(&self, cmd: Command) {
        self.cmd_queue.push(cmd)
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
                    self.change_selection(dir);
                },
                _ => {},
            }
            next = self.cmd_queue.pop_front();
        }
        super::super::RunState::MainMenu
    }
}
