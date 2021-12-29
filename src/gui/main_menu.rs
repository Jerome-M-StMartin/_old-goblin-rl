//Jerome M. St.Martin
//05/19/2021

use std::sync::{Arc, Mutex};
use std::any::Any;

use bracket_terminal::prelude::{BTerm, Point};

use super::look_n_feel::{ColorOption, Dir};
use super::drawable::Drawable;
use super::observer::{Observer, Observable};
use super::command::{Command, Commandable};
use super::user_input::{InputEvent, UserInput};

#[derive(PartialEq, Copy, Clone)]
pub enum Selection { NewGame, LoadGame, Quit }

pub struct MainMenu {
    pos: Point,
    selection: Mutex<Selection>,
    selection_made: Mutex<bool>,

    observer_id: usize,
    user_input: Arc<dyn Observable>,
}

impl MainMenu {
    pub fn new(user_input: Arc<UserInput>) -> Self {
        let new_id: usize;
        if let Ok(guard) = user_input.id_gen.lock() {
            new_id = guard.generate_observer_id();
        } else { panic!("Mutex poisoned, GUI::MainMenu::new()") };

        MainMenu {
            pos: Point {x:0,y:0},
            selection: Mutex::new(Selection::NewGame),
            selection_made: Mutex::new(false),
            observer_id: new_id,
            user_input,
        }
    }
    pub fn get_selection(&self) -> Option<Selection> {
        if let Ok(guard) = self.selection.lock() {
            Some(guard)
        } else {
            panic!("Mutex poisoned, noticed in MainMenu::get_selection().");
        };
        None
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
impl Commandable<MainMenu> for MainMenu {
    fn send(&self, cmd: Arc<dyn Command<MainMenu>>) {
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
impl Command<MainMenu> for ChangeSelectionCommand {
    fn execute(&self, main_menu: &MainMenu) {
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
impl Command<MainMenu> for SelectCommand {
    fn execute(&self, main_menu: &MainMenu) {
        if let Ok(mut guard) = main_menu.selection_made.lock() {
            *guard = true;
        }
    }
    fn as_any(&self) -> &dyn Any { self }
}
