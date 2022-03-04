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
