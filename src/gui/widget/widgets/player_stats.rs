use std::sync::Arc;

use bracket_lib::prelude::Point;

use super::super::{Widget, BoxType, WidgetElement, WIDGET_DATA};
use super::super::super::UserInput;

// Returns the observer_id of the widget, to allow for needed mutations to
// UserInput, such as making this newly constructed widget the Focus observer.
pub fn construct(user_input: &Arc<UserInput>) -> usize {
    //let (x_chars, y_chars) = ctx.get_char_size();
    let mut widget: Widget = Widget::new("PlayerStats",
                                         Point { x: 0, y: 0 },
                                         Point { x: 12, y: 5 },
                                         &user_input,
                                        );
            

    let widget_elements: &mut Vec<WidgetElement>;
    if let Ok(mut widget_data) = WIDGET_DATA.lock() {
        if let Some(elements) = widget_data.get_mut("PlayerStats") {
            widget_elements = elements;
            widget.with_these(widget_elements);
        }
    } else { panic!("Mutex poisoned in gui::widget::widgets::player_stats.rs") }

    widget.with_border(BoxType::THIN);
    widget.build()
}
