//Jerome M. St.Martin
//05/24/2021

use std::sync::{Arc, Mutex};
use std::any::Any;

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
    selection: Mutex<Selection>,
    selection_made: Mutex<bool>,

    observer_id: usize,
    user_input: Arc<dyn Observable>,
}

impl GameOver {
    pub fn new(user_input: Arc<UserInput>) -> Self {
        if let Ok(guard) = user_input.id_gen.lock() {
            GameOver {
                pos: Point {x:0,y:0},
                selection: Mutex::new(Selection::NewGame),
                selection_made: Mutex::new(false),
                observer_id: guard.generate_observer_id(),
                user_input: user_input.clone(),
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

    //Called by Observable when its data is dirty/
    //Calls .send() on a Command for self
    fn update(&self) {
        let observable = self.user_input.as_any().downcast_ref::<UserInput>();
        if let Some(user_input) = observable {
            if let Ok(guard) = user_input.input.read() {
                if let Some(input_event) = *guard {
                    match input_event {
                        InputEvent::HJKL(dir) | InputEvent::WASD(dir) => {
                            self.send(Arc::new(ChangeSelectionCommand::new(dir)));
                        },
                        InputEvent::ENTER => {
                            self.send(Arc::new(SelectCommand::new()));
                        }
                        _ => {},
                    }
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
    fn send(&self, cmd: Arc<dyn Command<GameOver>>) {
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
        if let Ok(mut guard) = main_menu.selection_made.lock() {
            *guard = true;
        }
    }
    fn as_any(&self) -> &dyn Any { self }
}
