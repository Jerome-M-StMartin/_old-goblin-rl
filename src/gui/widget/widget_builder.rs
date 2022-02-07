use bracket_lib::prelude::*;

pub enum BoxType { //determines if the widget border is drawn with '┓'(thick) or '┐'(thin).
    THIN,
    THICK,
}

pub struct WidgetElement {
    to_draw: String,
    color: RGB,
}
pub struct Widget {
    //Required fields on init
    name: String,
    position: Point,
    dimensions: Point,

    //Required after a few .with() calls
    elements: Box<Vec<WidgetElement>>, //to be drawn IN ORDER

    //Optional fields
    border: Option<BoxType>,
}

impl Widget {
    pub fn new<T: ToString>(name: T,
                            position: Point,
                            dimensions: Point,
                            elements: Vec) -> Self {

        Widget {
            name: name.to_string(),
            position,
            dimensions,
            elements: Box::new(elements),
            border: None,
        }
    }

    // --- BUILDER PATTERN ---
    pub fn with_border(mut self, box_type: BoxType) -> Self {
        self.border = Some(box_type);
    }

    pub fn with<T: ToString>(mut self, text: T) -> Self { //defaults color to WHITE
        self.elements.push(
            WidgetElement {
                to_draw: Box::new(text.to_string()),
                color: WHITE,
            }
        );
        self
    }

    pub fn with<T: ToString>(mut self, text: T, color: RGB) -> Self { //no default color, it must be specified
        self.elements.push(
            WidgetElement {
                to_draw: Box::new(text.to_string()),
                color,
            }
        );
        self
    }

    pub fn build(self) {
        super::widget_storage::add(self.name)
    }
    // --- END BUILDER PATTERN ---

    pub fn draw(self) {
        //TODO
    }
}
