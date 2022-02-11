use std::collections::HashMap;
use std::sync::Mutex;
use bracket_lib::prelude::BTerm;
use super::Widget;

lazy_static! {
    static ref WIDGET_STORAGE: Mutex<HashMap<String, Widget>> = Mutex::new(HashMap::new());
}

pub fn draw_widgets(ctx: &mut BTerm) {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        for widget in widget_map.values() {
            widget.draw(ctx);
        }
    } else {
        panic!("WIDGET_STORAGE Mutex was poisoned! (gui::widget::widget_storage.rs)")
    }
}

pub fn add<S: ToString>(widget: Widget) {
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        widget_map.insert(widget.name.to_string(), widget);
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
