/* Concept: An object owned by the GUI struct which accesses ECS data and displays that
 * data in the HUD dynamically.
 *
 */

use std::any::Any;
use std::collections::HashMap;

use bracket_terminal::prelude::{Point, BTerm};

pub struct WidgetStorage {
    data: HashMap<String, Box<dyn IsWidget>>, //arbitrarily chose 8, could only think of 6 but we'll see...
}

impl WidgetStorage {
    fn new() -> Self {
        WidgetStorage {
            data: HashMap::new(),
        }
    }

    fn tick(&self, ctx: &mut BTerm) { //draw_all(), basically
        for widget in self.data.values() {
            widget.draw(ctx);
        }
    }

    fn add<S: ToString>(&mut self, name: S, widget: Box<dyn IsWidget>) {
        self.data.insert(name.to_string(), widget);
    }

    fn rm(&mut self, widget_name: &str) -> Result<Box<dyn IsWidget>, String> {
        if let Some(widget) = self.data.remove(widget_name) {
            return Ok(widget);
        }
        Err("No such key found in WidgetStorage.data: HashMap.".to_string())
    }
}

//replaces gui::Drawable
pub trait IsWidget {
    fn draw(&self, ctx: &mut BTerm);
    fn move_to(&mut self, pos: Point);
    fn as_any(&self) -> &dyn Any;
}
