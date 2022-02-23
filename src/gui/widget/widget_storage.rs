use std::collections::HashMap;
use std::sync::Mutex;
use bracket_lib::prelude::BTerm;
use super::Widget;

lazy_static! {
    pub static ref WIDGET_STORAGE: Mutex<HashMap<String, Widget>> = Mutex::new(HashMap::new());
}

pub fn draw_all(ctx: &mut BTerm) {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        for widget in widget_map.values() {
            widget.draw(ctx);
        }
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}

pub fn get(widget_name: &str) -> Option<&Widget> {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        return widget_map.get(widget_name)
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}

pub fn add(widget: Widget, name: String) {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        widget_map.insert(name, widget);
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}

pub fn rm(widget_name: &str) -> Result<Widget, String> {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        if let Some(widget) = widget_map.remove(widget_name) {
            return Ok(widget);
        }
        return Err(format!("Key, {}, not found in WIDGET_STORAGE", widget_name));
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}

pub fn contains(widget_name: &str) -> bool {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        if let Some(_) = widget_map.get(widget_name) {
            return true
        }
        return false
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}
