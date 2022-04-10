use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use bracket_lib::prelude::{BTerm, DrawBatch};

use super::Widget;

lazy_static! {
    // Holds all the Widget structs created by super::widget_builder.
    pub static ref WIDGET_STORAGE: Mutex<HashMap<String, Arc<Widget>>> = Mutex::new(HashMap::new());
}

pub fn update_all() {
    if let Ok(mut widget_data) = super::WIDGET_DATA.lock() {
        if let Ok(mut widget_storage) = WIDGET_STORAGE.lock() {

            for key in widget_data.clone().keys() {
                if let Some(widget) = widget_storage.get_mut(key) {
                    if let Some(updated_elements) = widget_data.get_mut(key) {
                        
                        if let Ok(mut w_elems) = widget.elements.write() {
                            w_elems.clear();
                            w_elems.append(&mut updated_elements.clone());
                        }

                    }
                }
            }
        }
    }
}

pub fn draw_all(ctx: &mut BTerm, draw_batch: &mut DrawBatch) { //calls Commandable::process() and Widget::draw() on all widgets.
    if let Ok(widget_map) = WIDGET_STORAGE.lock() {
        for widget in widget_map.values() {
            widget.draw(ctx, draw_batch);
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
