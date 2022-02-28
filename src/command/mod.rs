use std::sync::Mutex;

use specs::World;

use super::RunState;
use super::gui::look_n_feel::Dir;
//-----------------------------------------------------
//---------------- COMMAND PATTERN --------------------
//-------- polymorphism via ENUMS this time! ----------
//-----------------------------------------------------


/* NOTES:
 * Due to each enum variant occupying the size of the largest variant, it is important
 * that if I eventually stick something big into a Command variant that I do so with a
 * reference to that big thing rather than a variant-owned value.
 *
 * If I need to ensure that a function can only operate on a specific enum variant, the way
 * to do this is to wrap a struct inside that variant, and check that struct for its type.
 * In rust, enum variants are NOT their own types, but structs are. Thus why this method is
 * required to do type-checking on variants. (e.g. see std::net::IpAddr)
 */

/* IDEA
 *
 * Problem that spawned the idea:
 * How do I pass a mutable borrow of the ECS World to the PlayerController,
 * or any other Commandable, so they can execute their commands?
 *
 * Solution:
 * Do not allow Commandables to execute their own commands!
 * Instead, have them place Commands received via ::send() into a queue,
 * and process this queue in an ECS System!
 *
 * Immediately Obvious Challenges:
 * How will the System know how to process each Command in the context
 * of that Command's target Commandable?
 */

pub enum Command {
    Grab,
    Move { dir: Dir },
    Select,
    Undo,
    Wait,
}

pub trait Commandable {
    //Implementors should have CommandQueue & CommandHistory fields.
    fn send(&self, command: Command); //store cmd in CommandQueue
    fn process(&self, ecs: &mut World, runstate: RunState) -> RunState; //execute each command in CommandQueue
    fn undo(&self) {} //Optional, requires CommandHistory field
}


// Command Storage for lazy processing/execution ---------
pub struct CommandQueue {
    queue: Mutex<Vec<Command>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        CommandQueue { queue: Mutex::new(Vec::new()), }
    }

    pub fn push(&self, cmd: Command) {
        if let Ok(mut queue_guard) = self.queue.lock() {
            queue_guard.push(cmd);
        } else { panic!("Mutex poisoned in command::CommandQueue."); };
    }

    pub fn rm_last(&self) -> Option<Command> {
        if let Ok(mut queue_guard) = self.queue.lock() {
            if !queue_guard.is_empty() {
                return queue_guard.pop();
            }
            
        } else { panic!("Mutex poisoned in command::CommandQueue."); };

        None
    }

    pub fn pop(&self) -> Option<Command> { //use this for true Queue-like behaviour
        if let Ok(mut queue_guard) = self.queue.lock() {
            if !queue_guard.is_empty() {
                return Some(queue_guard.remove(0)); //O(n), but preserves order
            }
        } else { panic!("Mutex poisoned in command::CommandQueue."); };

        None
    }
    
    pub fn clear(&self) {
        if let Ok(mut queue_guard) = self.queue.lock() {
            queue_guard.clear()
        } else { panic!("Mutex poisoned in command::CommandQueue."); };
    }

    pub fn iter(&self) -> CommandQueueIter<Command> {
        if let Ok(queue_guard) = self.queue.lock() {
            return CommandQueueIter {
                guard: queue_guard,
            }
        }
        panic!("Mutex poisoned. (command::CommandQueue::into_iter())");
    }
}

pub struct CommandQueueIter<'a, Command> {
    guard: std::sync::MutexGuard<'a, Vec<Command>>,
}

impl<'a, 'b: 'a, Command: 'a> IntoIterator for &'b CommandQueueIter<'a, Command> {
    type Item = &'a Command;
    type IntoIter = std::slice::Iter<'a, Command>;
    
    fn into_iter(self) -> std::slice::Iter<'a, Command> {
        self.guard.iter()
    }
}
//-------------------------------------------------------

//Funtionality for undo-ability of Commands --------------
pub struct CommandHistory {
    history: Mutex<Vec<Command>>,
}

//Only the owning Commandable obj should ever be calling these.
impl CommandHistory {
    pub fn new() -> Self {
        CommandHistory { history: Mutex::new(Vec::new()), }
    }

    pub fn push(&mut self, cmd: Command) {
        if let Ok(mut history) = self.history.lock() {
            history.push(cmd);
        }
    }

    pub fn pop(&mut self) -> Option<Command> {
        if let Ok(mut history) = self.history.lock() {
            return history.pop();
        };

        None
    }
}//-------------------------------------------------------

