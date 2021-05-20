//Jerome M. St.Martin
//Node Menu Project
//03/06/2021

//This obj is the Observable of an Observer Pattern used to pass user input
//around to the currently active interface/object, called the "focus".
//
//Each Observer has read-only access to the InputState.
//The Observers should receieve a call to Observer::update() whenever the InputState mutates.
//Each Observer can then translate any subset of the InputState into Commands (see Command Pattern)
//which will then control/mutate the user interface of that Observer object. This way each Observer
//holds its own internal set of Commands without telling any outside objects what they are. If
//input is invalid for the "focus" Observer, the corresponding input in the InputState is simply
//ignored by the focus Observer.

use super::look_n_feel::Dir;
use super::observer::{IdGenerator, Observable, Observer};
use bracket_terminal::prelude::{BTerm, Point, VirtualKeyCode};
use std::any::Any;
use std::sync::{RwLock, Weak, Arc};

pub struct UserInput {
    pub id_gen: IdGenerator,
    observers: RwLock<Vec<Weak<dyn Observer>>>, //focus is at index 0
    pub input: RwLock<Option<InputEvent>>,
    focus_id: RwLock<Option<usize>>,
}

#[derive(Clone, Copy)]
pub enum InputEvent {
    MOUSE(Point),
    CURSOR(Point),
    WASD(Dir),
    HJKL(Dir),
    TOOLTIPS,
    ESC,
    ENTER,
}

impl UserInput {
    pub fn new() -> Self {
        UserInput {
            id_gen: IdGenerator::new(),
            observers: RwLock::new(Vec::new()),
            input: RwLock::new(None),
            focus_id: RwLock::new(None),
        }
    }

    pub fn transcribe_input(&self, ctx: &BTerm) -> bool { //use return val to control observer notification
        let mut dirty: bool = false;
        let mut input: Option<InputEvent>;
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::W => input = Some(InputEvent::WASD(Dir::UP)),
                VirtualKeyCode::S => input = Some(InputEvent::WASD(Dir::DOWN)),
                VirtualKeyCode::A => input = Some(InputEvent::WASD(Dir::LEFT)),
                VirtualKeyCode::D => input = Some(InputEvent::WASD(Dir::RIGHT)),

                VirtualKeyCode::K => input = Some(InputEvent::HJKL(Dir::UP)),
                VirtualKeyCode::J => input = Some(InputEvent::HJKL(Dir::DOWN)),
                VirtualKeyCode::H => input = Some(InputEvent::HJKL(Dir::LEFT)),
                VirtualKeyCode::L => input = Some(InputEvent::HJKL(Dir::RIGHT)),

                VirtualKeyCode::Tab => {
                    let next_id = &self.next_observer_id();
                    if *next_id == 0 {
                        self.focus_id.set(None);
                    } else {
                        self.focus_id.set(Some(next_id.clone()));
                    }
                }
                VirtualKeyCode::T => input = Some(InputEvent::TOOLTIPS),
                VirtualKeyCode::Escape => input = Some(InputEvent::ESC),
                VirtualKeyCode::Return => input = Some(InputEvent::ENTER),

                _ => {
                    input = None;
                    if let Ok(unlocked_input) = self.input.write() {
                        unlocked_input = input;
                    }
                    return dirty;
                }
            }
        } else {
            input = None;
            self.input.set(input);
            return dirty;
        };

        dirty = true;
        self.input.set(input);
        return dirty;
    }

    //Return next observer id after popping next_focus from observers vec and moving it to the
    //front (observers[0] is the current focus).
    fn next_observer_id(&self) -> usize {
        let mut observers = self.observers.borrow_mut();
        let mut next_focus_id: usize = 0;
        if let Some(next_focus_ptr) = observers.pop() {
            if let Some(next_focus) = next_focus_ptr.upgrade() {
                next_focus.setup_cursor();
                next_focus_id = next_focus.id();
                let strong_ptr = next_focus;
                let weak_ptr = Arc::downgrade(&strong_ptr);
                observers.insert(0, weak_ptr);
            }
        }

        return next_focus_id;
    }
}

impl Observable for UserInput {
    fn notify_observers(&self) {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut idx: usize = 0;
        for observer_ptr in self.observers.borrow().iter() {
            if let Some(observer) = observer_ptr.upgrade() {
                observer.update();
            } else {
                //queue lazy removal of dropped observers
                to_remove.push(idx);
            }
            idx += 1;
        }

        //lazy removal of dropped observers:
        for idx in to_remove.into_iter() {
            self.observers.borrow_mut().swap_remove(idx); //swap_remove() does not preserve order but is O(1).
        }
    }

    fn notify_focus(&self) {
        let observer = &self.observers.borrow()[0];
        if let Some(focus) = observer.upgrade() {
            focus.update();
        }
    }

    //Called by Observer trait objects who want to be notified by this Observable.
    fn add_observer(&self, to_add: Weak<RwLock<dyn Observer>>) {
        if let Some(observer) = to_add.upgrade() {
            *self.focus_id.borrow_mut() = Some(observer.id());
        }
        self.observers.borrow_mut().push(to_add);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
