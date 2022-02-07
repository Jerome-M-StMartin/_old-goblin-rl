use bracket_lib::prelude::RGB;
use logstore::*;

mod logger;
mod logstore;

pub use logger::*;
pub use logstore::{clear_log, log_display};

/* Usage
* let mut block = TextBlock::new(1, 46, 79, 58);
block.print(&gamelog::log_display());
block.render(&mut bracket-lib::BACKEND_INTERNAL.lock().consoles[0].console);
*/

pub struct LogFragment {
    pub color : RGB,
    pub text : String
}