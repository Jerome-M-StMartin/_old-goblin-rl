//use std::any::Any;

mod widget_builder;
pub mod widget_storage;

//-- Widgets --
pub mod main_menu;
pub mod game_over;
// -- --
pub use widget_builder::*;
pub use widget_storage::*;

/*replaces gui::Drawable
* Anything that can be represented by/in a Widget must implement this trait.
* In doing so, each dyn WidgetData tells the GUI exactly how it is to be drawn
* within its representing Widget.
*/

pub trait WidgetData {
    //The order of elements in this Vec determines the order in which they will be drawn.
    fn as_widget_elements(&self) -> Vec<WidgetElement>;
}

use bracket_lib::prelude::{RGB, RED, GREEN, BLUE};
use super::super::components::Stats;
use std::cmp::max;
impl WidgetData for Stats {

    fn as_widget_elements(&self) -> Vec<WidgetElement> {
        let mut result = Vec::new();
        let max = max( self.max_hp, max(self.max_fp, self.max_mp) );

        let mut s = "".to_string();
        for i in 0..max {
            if i < self.hp { s.push('♡'); }
            else if i < self.max_hp { s.push('∅'); }
        }

        result.push( WidgetElement::new(s.clone(), RGB::named(RED)) );
        s.clear();

        for i in 0..max {
            if i < self.fp { s.push('♦'); }
            else if i < self.max_fp { s.push('∅'); }
        }

        result.push( WidgetElement::new(s.clone(), RGB::named(GREEN)) );
        s.clear();

        for i in 0..max {
            if i < self.mp { s.push('⏾'); }
            else if i < self.max_mp { s.push('∅'); }
        }

        result.push( WidgetElement::new(s.clone(), RGB::named(BLUE)) );

        result
    }
}
