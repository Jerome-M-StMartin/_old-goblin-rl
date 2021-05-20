//Jerome M. St.Martin
//Node Menu Project
//12/18/2020

//OBSERVER DESIGN PATTERN
//Many Observers per Subject/Observable

use std::any::Any;
use std::collections::HashSet;
use std::sync::{RwLock, Weak};

pub struct IdGenerator {
    used_ids: RwLock<HashSet<usize>>,
}
impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator {
            used_ids: RwLock::new(HashSet::new()),
        }
    }

    //Guaranteed to return a unique usize for this session.
    pub fn generate_observer_id(&self) -> usize {
        let mut new_id: usize = rand::random();
        let mut used_ids: HashSet<usize>;
        if let Ok(used_ids_guard) = self.used_ids.write() {
            used_ids = *used_ids_guard;
        }
        while used_ids.contains(&new_id) {
            new_id = rand::random();
        }
        used_ids.insert(new_id);
        new_id
    }
}

pub trait Observer {
    //Each implementor of Observer must store a unique observer id,
    //so each Observable needs either: access to a shared IdGenerator,
    //or its own IdGenerator. Observers must not be shared between
    //Observables with separate IdGenerators.
    fn id(&self) -> usize;
    fn update(&self);
    fn setup_cursor(&self);
}

//Implementors of the Observable trait should probably be placed behind a Mutex,
//else Rust will worry about data-racing and make it impossible for the Observers
//to observe this Observable.
pub trait Observable {
    fn notify_observers(&self); //<-implement lazy removal of dropped observers in here.
    fn notify_focus(&self);
    fn add_observer(&self, to_add: Weak<RwLock<dyn Observer>>);
    fn as_any(&self) -> &dyn Any;
}

/* EXAMPLE Observable Trait IMPLEMENTATION:
impl Observable for MyStruct {
    fn notify(&self, to_notify: &Vec<Box<dyn Observer>>) {
        for observer in to_notify {
            observer.update();
        }
    }

    fn add_observer(to_add: Box<dyn Observer>, to_notify: &mut Vec<Box<dyn Observer>>) {
        to_notify.push(to_add);
    }

    fn rm_observer(&self, to_remove: &Box<dyn Observer>, to_notify: &mut Vec<Box<dyn Observer>>) {
        let mut to_remove_idx = 0;
        for observer in to_notify.iter() {
            if observer.id() == to_remove.id() {
                break;
            }
            to_remove_idx += 1;
        }

        //swap_remove() used over remove() for O(1) runtime.
        //Currently, the order of Observers in this vec doesn't matter,
        //if this changes remove() will have to be used instead.
        to_notify.swap_remove(to_remove_idx);
    }
}*/
