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
use std::sync::{Mutex, RwLock, Arc, Weak};

use bracket_lib::prelude::{BTerm, Point, VirtualKeyCode};

use super::gui;

use gui::look_n_feel::Dir;
use gui::observer::{IdGenerator, Observable, Observer};

//This struct should have only one instance; alias via Arc<>.
pub struct UserInput {
    pub id_gen: Mutex<IdGenerator>,
    pub input: RwLock<Option<InputEvent>>,
    selection: RwLock<Option<u8>>, //allows Widgets to communicate user selection to back-end
    focus_id: Mutex<Option<usize>>, //focus is variable & only Observer that gets .update() calls
    observers: Mutex<Vec<Weak<dyn Observer>>>, //focus is at index 0
}

#[derive(Clone, Copy)]
pub enum InputEvent {
    MOUSE(Point),
    CURSOR(Point),
    WASD(Dir),
    HJKL(Dir),
    SPACE,
    TOOLTIPS,
    ESC,
    ENTER,
}

impl UserInput {
    pub fn new() -> Self {
        UserInput {
            id_gen: Mutex::new(IdGenerator::new()),
            input: RwLock::new(None),
            selection: RwLock::new(None),
            focus_id: Mutex::new(None),
            observers: Mutex::new(Vec::new()),
        }
    }

    pub fn tick(&self, ctx: &BTerm) {
        if Self::transcribe_input(&self, ctx) {
            Self::notify_focus(&self);
        }
    }

    fn transcribe_input(&self, ctx: &BTerm) -> bool { //use bool to control observer notification
        
        //Clear past selection, if any.
        if let Ok(mut selection) = self.selection.write() {
            *selection = None;
        } else { panic!("RwLock poisoned! (user_input::transcribe_input"); }

        //Read user input
        let mut new_input: Option<InputEvent> = None;
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::W => new_input = Some(InputEvent::WASD(Dir::UP)),
                VirtualKeyCode::S => new_input = Some(InputEvent::WASD(Dir::DOWN)),
                VirtualKeyCode::A => new_input = Some(InputEvent::WASD(Dir::LEFT)),
                VirtualKeyCode::D => new_input = Some(InputEvent::WASD(Dir::RIGHT)),

                VirtualKeyCode::K => new_input = Some(InputEvent::HJKL(Dir::UP)),
                VirtualKeyCode::J => new_input = Some(InputEvent::HJKL(Dir::DOWN)),
                VirtualKeyCode::H => new_input = Some(InputEvent::HJKL(Dir::LEFT)),
                VirtualKeyCode::L => new_input = Some(InputEvent::HJKL(Dir::RIGHT)),

                VirtualKeyCode::Tab => { //Change Focus
                    let next_id = self.next_observer_id();
                    if let Ok(mut guard) = self.focus_id.lock() {
                        *guard = Some(next_id);
                    }
                }
                VirtualKeyCode::T => new_input = Some(InputEvent::TOOLTIPS),
                VirtualKeyCode::Escape => new_input = Some(InputEvent::ESC),
                VirtualKeyCode::Return => new_input = Some(InputEvent::ENTER),
                _ => {}
            }
        };

        if let Ok(mut input) = self.input.write() {
            *input = new_input;

            if input.is_some() {
                return true;
            }
        }
        
        false
    }

    //Return next observer id after popping next_focus from observers vec and moving it to the
    //front. (observers[0] should always be the current focus). Popped observer is never re-added
    //to observers vec if its Arc::upgrade fails, which is an easy lazy-removal implementation.
    fn next_observer_id(&self) -> usize {
        let mut focus_id: usize = 0;
        //obtain lock on observer-vec Mutex
        if let Ok(guard) = self.observers.lock() {
            let mut observers = guard;

            //grab next Weak<Observer> from the observer-vec & upgrade to Arc<>
            if let Some(next_focus_weak) = observers.pop() {
                if let Some(next_focus) = next_focus_weak.upgrade() {

                    //set & lite-initialize new focus id, insert at head of observer-vec
                    focus_id = next_focus.id();
                    if let Ok(mut guard) = self.focus_id.lock() {
                        *guard = Some(focus_id);
                    }
                    next_focus.become_focus();
                    observers.insert(0, Arc::downgrade(&next_focus));
                    dbg!("Next focus: {}", next_focus.name());//----------------------------------------Debugging
                }
            }
        }

        //return the new focus_id, which was the next observer when this fn was called
        focus_id
    }

    pub fn set_focus(&self, observer_id: usize) {
        let mut idx = 0;

        //obtain lock on observer-vec'
        if let Ok(mut observers) = self.observers.lock() {
            //iterate over each observer
            for observer_weak in observers.iter() {
                if let Some(observer) = observer_weak.upgrade() {

                    //check if it's the target
                    if observer.id() == observer_id {
                        let new_focus = observers.swap_remove(idx); //O(1) but not in-place
                        //let new_focus = observers.remove(idx);
                        observers.insert(0, new_focus);
                        
                        //obtain a Mutex lock & set new focus
                        if let Ok(mut focus) = self.focus_id.lock() {
                            *focus = Some(observer_id);
                        } else { panic!("Mutex poisoned! (UserInput::set_focus())"); }

                        observer.become_focus();
                        break;
                    }
                    idx += 1;
                }
            }
        } else { panic!("Mutex poisioned! (UserInput::set_focus())"); }
    }

    pub fn set_focus_selection(&self, new_selection: Option<u8>) {
        if let Ok(mut selection) = self.selection.write() {
            *selection = new_selection;
            return
        }
        panic!("Mutex poisoned! (user_input.set_selection())");
    }

    pub fn get_focus_selection(&self) -> Option<u8> {
        if let Ok(selection) = self.selection.read() {
            return *selection
        }
        panic!("Mutex poisoned! (user_input.get_focus_selection())");
    }

    pub fn generate_id(&self) -> usize {
        if let Ok(id_gen) = self.id_gen.lock() {
            return id_gen.generate_observer_id();
        }
        panic!("Unable to get ObserverID; was the id_gen Mutex poisoned???");
    }
}

impl Observable for UserInput {
    fn notify_observers(&self) {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut idx: usize = 0;

        if let Ok(guard) = self.observers.lock() {
            let mut observers = guard;
            for weak_observer in observers.iter() {
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
                    observers.swap_remove(idx); //swap_remove() does not preserve order but is O(1).
                    //observers.remove(idx);
                }
            }
   
        } else { panic!("Mutex was poisoned in user_input::mod.rs::notify_observers()"); }

    }

    fn notify_focus(&self) {
        let mut try_again: bool = false;
        if let Ok(mut observers) = self.observers.lock() {
            let weak_focus = observers[0].clone();

            if let Some(focus) = weak_focus.upgrade() {
                focus.update();
               
            } else if !observers.is_empty() { //upgrade failed, so rm this observer and re-call this fn
                //lazily delete the dropped observer pointer
                observers.remove(0);
                try_again = true;
            }
        } else { panic!("Mutex poisoned! (UserInput::notify_focus())"); }

        if try_again { 
            //re-set focus to self.focus_id, if it's Some
            if let Ok(focus_id) = self.focus_id.lock() {
                if let Some(id) = *focus_id {
                    self.set_focus(id);
                }
            }

            //re-call this fn
            self.notify_focus();
        }
    }

    //Called by Observer trait objects who want to be notified by this Observable.
    fn add_observer<T>(&self, to_add: &Arc<T>)
        where T: 'static + Observer {
        
        let as_observer: Weak<T> = Arc::downgrade(to_add);
        
        if let Ok(mut guard) = self.observers.lock() {
            guard.push(as_observer);
        }
    }
    /*fn add_observer(&self, to_add: &Arc<dyn Observer>) {
        if let Ok(mut guard) = self.observers.lock() {
            guard.push(Arc::downgrade(to_add));
        }
    }*/
    /*fn add_observer(&self, to_add: Weak<dyn Observer>) {
        if let Ok(mut guard) = self.observers.lock() {
            guard.push(to_add);
        }
    }*/

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/*DEPRECIATED - NO LONGER STORING DATA IN THE 'PLAYER' COMPONENT, IT IS ONLY AN ECS MARKER
 *
 * //Serde requires a Default method so it can populate the Player Component's
//Arc<UserInput> field upon Deserialization. This is just to satisfy rust's
//memory safety requirements, a proper reference to the 'real' UserInput
//struct must be fed to the Player Component after serialization occurrs.
impl Default for UserInput {
    fn default() -> Self {
        UserInput {
            id_gen: Mutex::new(IdGenerator::new()),
            observers: Mutex::new(Vec::new()),
            input: RwLock::new(Option::None),
            focus_id: Mutex::new(None),
        }
    }
}*/

/*Pre-threadsafe version:
pub struct UserInput {
    pub id_gen: IdGenerator,
    observers: RefCell<Vec<Weak<dyn Observer>>>, //focus is at index 0
    pub input: Cell<Option<InputEvent>>,
    focus_id: Cell<Option<usize>>, //focus is variable & is the only Observer that gets .update() calls from UserInput.
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
            self.input.set(input); //local input == None, in this case
            return dirty;
        };

        dirty = true;
        self.input.set(input);
        return dirty;
    }

    //Return next observer id after popping next_focus from observers vec and moving it to the
    //front. (observers[0] should always be the current focus). Popped observer is never re-added
    //to observers vec if its Rc::upgrade fails, which is an easy lazy-removal implementation.
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

    pub fn set_focus(&self, observer_id: usize) {
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
*/


