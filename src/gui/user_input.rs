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

use std::any::Any;
use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};

use bracket_terminal::prelude::{BTerm, Point, VirtualKeyCode};

use super::look_n_feel::Dir;
use super::observer::{IdGenerator, Observable, Observer};

pub struct UserInput {
    pub id_gen: IdGenerator,
    observers: RefCell<Vec<Weak<dyn Observer>>>, //focus is at index 0
    pub input: Cell<Option<InputEvent>>,
    focus_id: Cell<Option<usize>>,
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
            observers: RefCell::new(Vec::new()),
            input: Cell::new(None),
            focus_id: Cell::new(None),
        }
    }

    pub fn transcribe_input(&self, ctx: &BTerm) -> bool { //use returned bool to control observer notification
        let mut dirty: bool = false;
        let mut input: Option<InputEvent> = None;
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
                    let next_id = self.next_observer_id();
                    self.focus_id.set(Some(next_id));
                }
                VirtualKeyCode::T => input = Some(InputEvent::TOOLTIPS),
                VirtualKeyCode::Escape => input = Some(InputEvent::ESC),
                VirtualKeyCode::Return => input = Some(InputEvent::ENTER),

                _ => {
                    self.input.set(input);
                    return dirty;
                }
            }
        } else {
            self.input.set(input);
            return dirty;
        };

        dirty = true;
        self.input.set(input);
        return dirty;
    }

    //Return next observer id after popping next_focus from observers vec and moving it to the
    //front. (observers[0] should always be the current focus).
    fn next_observer_id(&self) -> usize {
        let mut focus_id: usize = 0;
        let mut observers = self.observers.borrow_mut();
        if let Some(next_focus_weak) = observers.pop() {
            if let Some(next_focus) = next_focus_weak.upgrade() {
                focus_id = next_focus.id();
                self.focus_id.set(Some(focus_id));
                next_focus.setup_cursor();
                observers.insert(0, Rc::downgrade(&next_focus));
            }
        }
        focus_id
    }

    fn set_focus(&self, observer_id: usize) {
        let mut idx = 0;
        let mut observers = self.observers.borrow_mut();
        for observer_weak in observers.clone().iter() {
            if let Some(observer) = observer_weak.upgrade() {
                if observer.id() == observer_id {
                    let new_focus = observers.swap_remove(idx);
                    observers.insert(0, new_focus);
                    self.focus_id.set(Some(observer_id));
                }
                idx += 1;
            }
        }
    }
}

impl Observable for UserInput {
    fn notify_observers(&self) {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut idx: usize = 0;

        for weak_observer in self.observers.borrow_mut().iter() {
            if let Some(observer) = weak_observer.upgrade() {
                observer.update();
            } else {
                to_remove.push(idx);
            }
            idx += 1;
        }

        //lazy removal of dropped observers
        if !to_remove.is_empty() {
            for idx in to_remove.into_iter() {
                self.observers.borrow_mut().swap_remove(idx); //swap_remove() does not preserve order but is O(1).
            }
        }
    }

    fn notify_focus(&self) {
        let weak_focus = &self.observers.borrow()[0];
        if let Some(focus) = weak_focus.upgrade() {
            focus.update();
        }
    }

    //Called by Observer trait objects who want to be notified by this Observable.
    fn add_observer(&self, to_add: Weak<dyn Observer>) {
        if let Some(_) = to_add.upgrade() {
            self.observers.borrow_mut().push(to_add);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
