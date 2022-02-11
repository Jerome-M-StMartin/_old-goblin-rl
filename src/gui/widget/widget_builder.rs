use std::sync::Arc;

use bracket_lib::prelude::*;

use super::super::look_n_feel::Dir;
use super::super::observer::Observer;
use super::super::user_input::{UserInput, InputEvent};
use super::super::command::{Command, Commandable, CommandQueue};                
use super::super::super::{World, RunState};

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

    //Observer Pattern Fields
    observer_id: usize,
    user_input: Arc<UserInput>,
    focus_element_idx: Option<u8>,

    //Command Pattern Fields
    cmd_queue: CommandQueue,
}

impl Widget {
    pub fn new<T: ToString>(name: T,
                            position: Point,
                            dimensions: Point,
                            elements: Vec,
                            user_input: Arc<UserInput>) -> Self {

        Widget {
            name: name.to_string(),
            position,
            dimensions,
            elements: Box::new(elements),
            border: None,
            observer_id: user_input.generate_id(),
            user_input,
            focus_element_idx: None,
            cmd_queue: CommandQueue::new(),
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

    pub fn draw(self, &mut ctx: BTerm) {
        /*TODO:
         * Change to accept textbuilder (and maybe other structs) as argument,
         * such that all widgets can be drawn to a single buffer which is drawn
         * to the context only once per tick, instead of multiple draws for the
         * many widgets.
         */
        let (x, y, w, h) = (self.position.x + 1,
                         self.position.y + 1,
                         self.dimensions.x - 2,
                         self.dimensions.y - 2);

        //check pos/dim for validity
        if x < 0 || y < 0 || w < 0 || h < 0 { return };
        if x > ctx.width || y > ctx.height || w > ctx.width || h > ctx.height { return; };

        let mut draw_batch = DrawBatch::new();

        let mut textblock = TextBlock::new(x, y, w, h);
        let mut textbuilder = TextBuilder::empty();

        let mut idx = 0;
        for element in self.elements.iter() {
            let mut color: RGB = WHITE;
            if let Some(focus_idx) = self.focus_element_idx {
                if focus_idx == idx { color = MAGENTA; };
            }
            textbuilder.color(color);
            textbuilder.append(element);
            textbuilder.ln();
        }

        textbuilder.reset(); //unnecessary until I pass-by-&mut the textbuilder to this fn
        //TODO: draw border
        textblock.print(&textbuilder).expect("Text too long. (Widget.draw())");
        textblock.render_to_draw_batch(draw_batch);
        draw_batch.submit(0).expect("Batch error in Widget.draw()");
        render_draw_buffer(ctx).expect("Render error in Widget.draw()");
    }
}

impl Observer for Widget {
    fn id(&self) -> usize {
        self.observer_id
    }

    fn update(&self) {
        if let Ok(input) = self.user_input.input.read() {
            if let Some(input_event) = *input {
                let cmd_option = match input_event {
                    InputEvent::WASD(dir) |
                    InputEvent::HJKL(dir) => { Some(Command::Move{dir}) },
                    _ => { None },
                };

                if let Some(cmd) = cmd_option {
                    self.send(cmd);
                }
            }
        }
    }

    fn setup_cursor(&self) {}

    fn name(&self) -> &str { self.name }
}

impl Commandable for Widget {
    fn send(&self, cmd: Command) {
        self.cmd_queue.push(cmd);
    }

    fn process(&self, _ecs: &mut World) -> RunState {
        let mut runstate = RunState::AwaitingInput;
        for cmd in self.cmd_queue.iter() {
            match cmd {
                Command::HJKL(dir) => {
                    match dir {
                        Dir::UP => { self.focus_element_idx -= 1 },
                        Dir::DOWN => { self.focus_element_idx += 1 },
                        Dir::LEFT | Dir::RIGHT => {},
                    }
                },
                Command::Select => {
                    //TODO
                    println!("'{}' Widget: Element {} selected.", self.name, self.focus_element_idx);
                }
            }
        };

        runstate
    }
}
