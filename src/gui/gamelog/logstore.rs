use std::sync::Mutex;
use super::LogFragment;
use bracket_lib::prelude::TextBuilder;

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

pub fn append_fragment(fragment: LogFragment) {
    if let Ok(mut log) = LOG.lock() {
        log.push(vec![fragment]);
    }
}

pub fn append_entry(fragments: Vec<LogFragment>) {
    if fragments.is_empty() { return; }
    LOG.lock().unwrap().push(fragments);
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
}

pub fn log_display() -> TextBuilder {
    let mut buf = TextBuilder::empty();

    LOG.lock().unwrap().iter().rev().take(12).for_each(|log| {
        log.iter().for_each(|frag| {
            buf.fg(frag.color);
            buf.line_wrap(&frag.text);
        });
        buf.ln();
    });

    buf
}
