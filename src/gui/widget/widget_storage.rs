use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use bracket_lib::prelude::BTerm;

use super::Widget;

lazy_static! {
    pub static ref WIDGET_STORAGE: Mutex<HashMap<String, Arc<Widget>>> = Mutex::new(HashMap::new());
}

pub fn draw_all(ctx: &mut BTerm) { //calls Commandable::process() and Widget::draw() on all widgets.
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        for widget in widget_map.values() {
            widget.draw(ctx);
        }
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage::draw_all())")
    }
}

pub fn add(widget: Arc<Widget>) {
    if let Ok(mut widget_map) = WIDGET_STORAGE.lock() {
        widget_map.insert(widget.name(), widget);
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage::add())")
    }
}

pub fn rm(widget_name: &str) -> Result<Arc<Widget>, String> {
    if let Ok(mut widget_map) = WIDGET_STORAGE.lock() {
        if let Some(widget) = widget_map.remove(widget_name) {
            return Ok(widget);
        }
        return Err(format!("Key, {}, not found in WIDGET_STORAGE", widget_name));
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage::rm())")
    }
}

pub fn contains(widget_name: &str) -> bool {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        return widget_map.contains_key(widget_name);
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage::contains())")
    }
}
