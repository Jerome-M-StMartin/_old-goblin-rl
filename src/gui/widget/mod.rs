//use std::any::Any;

mod widget_builder;
pub mod widget_storage;

//-- Widgets --
pub mod main_menu;
// -- --
pub use widget_builder::*;
pub use widget_storage::*;

/*replaces gui::Drawable
* Anything that can be represented by/in a Widget must implement this trait.
* In doing so, each dyn WidgetData tells the GUI exactly how it is to be drawn
* within its representing Widget.
*/
/*pub trait WidgetData {
    fn as_widget_elements(&self) -> Vec<WidgetElement>;
    fn as_any(&self) -> &dyn Any;
}*/

/*
* as_any is always implemented as such:
* 
* fn as_any(&self) -> &dyn Any { self }
*
* Can't have default implementation here since a dyn Self isn't sized.
*/
