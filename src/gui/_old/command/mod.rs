//Jerome M. St.Martin
//12/07/2020

//COMMAND PATTERN

/* How To Use:
 * 1.) Create new command obj (struct that implements Command trait).
 * 2.) Make them public so other things can create and give ownership of them.
 * 3.) Have some input handler somewhere generate these command objects and send
 *     them where they need to go.
 *
 * In this way, each type that needs to use this Command Pattern has a
 * unique set of commands that apply only to itself. There are no generic
 * shared commands.
 */

/* Deeper Explanation for <T> and <dyn Command>:
 *
 * T, in this context, represents the type of object that this specific
 * command applies to.
 *
 * dyn Command, in this context, represents one of any number of commands
 * that are implemented for a specific T.
 *
 * So target_instance is an immutable borrow of an object of type T.
 */

use std::any::Any;
use std::sync::{Arc, Mutex};

pub trait Command<T> {
    fn execute(&self, target_instance: &T);
    fn as_any(&self) -> &dyn Any;
    fn reverse_me(&mut self) {} //override if this command is being sent to a Commandable with a CommandHistory.
}

pub trait Commandable<T> {
    fn send(&self, cmd: Arc<dyn Command<T>>);
}

//A Commandable with a CommandHistory should implement a fn reverse_cmd(cmd) method on itself,
//such that the commands in CommandHistory can be executed as normal without looking into the
//history of what was changed. Thus the commands themselves contain the deltas needed to revert
//their previous execution.
pub struct CommandHistory<T> {
    hist: Mutex<Vec<Arc<dyn Command<T>>>>,
}

impl<T> CommandHistory<T> {
    pub fn new() -> Self {
        //CommandHistory { hist: RefCell::new(Vec::new()) }
        CommandHistory { hist: Mutex::new(Vec::new()) }
    }

    pub fn push(&self, cmd: impl Command<T> + 'static) {
        if let Ok(mut guard) = self.hist.lock() {
            guard.push(Arc::new(cmd));
        }
    }

    pub fn pop(&self) -> Result<Arc<dyn Command<T>>, &str> {
        if let Ok(mut guard) = self.hist.lock() {
            if let Some(last_cmd) = guard.pop() {
                return Ok(last_cmd);
            }
        }
        Err("Command History vec is empty.")
    }
}
