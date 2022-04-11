use std::sync::Mutex;
use std::collections::HashMap;

mod widget_builder;
pub mod widget_storage;
pub mod widgets;

pub use widget_builder::*;
pub use widget_storage::*;
pub use widgets::*;

lazy_static! {
    // Holds returns from Widgetable::as_widget_elements(), as defined by the below trait.
    pub static ref WIDGET_DATA: Mutex<HashMap<String, Vec<WidgetElement>>> = Mutex::new(HashMap::new());
}

pub fn store_widget_data<T: ToString>(key: T, val: Vec<WidgetElement>) {
    if let Ok(mut map) = WIDGET_DATA.lock() {
        map.insert(key.to_string(), val);
    } else { panic!("Mutex poisoned in gui::widget::mod.rs"); }
}

/* Widgetable replaces gui::Drawable :
* Anything that can be represented by/in a Widget must implement this trait.
* In doing so, each dyn Widgetable tells the GUI exactly how it is to be drawn
* within its representing Widget. */

pub trait Widgetable {
    //The order of elements in this Vec determines the order in which they will be drawn.
    //Top-to-bottom, my_vec[0] is top, my_vec[my_vec.len() - 1] is bottom.
    fn as_widget_elements(&self) -> Vec<WidgetElement>;
}

use bracket_lib::prelude::{RGB, RED, GREEN, BLUE};
use super::super::components::Stats;
use std::cmp::max;
impl Widgetable for Stats {

    fn as_widget_elements(&self) -> Vec<WidgetElement> {
        let mut result = Vec::new();
        let max = max( self.max_hp, max(self.max_fp, self.max_mp) );

        let mut s = "[".to_string();
        for i in 0..max {
            if i < self.hp { s.push('♥'); }
            else if i < self.max_hp { s.push('•'); }
        }
        s.push(']');
        result.push( WidgetElement::new(s.clone(), RGB::named(RED)) );
        s.clear();
    
        s.push('<');
        for i in 0..max {
            if i < self.fp { s.push('♦'); }
            else if i < self.max_fp { s.push('•'); }
        }
        s.push('>');
        result.push( WidgetElement::new(s.clone(), RGB::named(GREEN)) );
        s.clear();

        s.push('(');
        for i in 0..max {
            if i < self.mp { 
                match i % 2 {
                    1 => s.push('▲'),
                    0 => s.push('▼'),
                    _ => {},
                };
            } else if i < self.max_mp { s.push('•'); }
        }
        s.push(')');
        result.push( WidgetElement::new(s.clone(), RGB::named(BLUE)) );

        result
    }
}
