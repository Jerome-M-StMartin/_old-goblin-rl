use std::sync::{Arc, RwLock};

use bracket_lib::prelude::{BTerm, DrawBatch, Point, RGB,  RGBA, WHITE, MAGENTA,
                           TextBlock, TextBuilder, Rect, ColorPair};
use specs::World;

use crate::gui::Observable;

use super::super::look_n_feel::Dir;
use super::super::observer::Observer;
use super::super::user_input::{UserInput, InputEvent};
use super::super::command::{Command, Commandable, CommandQueue};                
use super::super::super::RunState;

pub enum BoxType { //determines if the widget border is drawn with '┓'(thick) or '┐'(thin).
    THIN,
    THICK,
}

#[derive(Debug, Clone)]
pub struct WidgetElement {
    to_draw: String,
    color: RGB,
}
impl WidgetElement {
    pub fn new(s: String, c: RGB) -> WidgetElement {
        WidgetElement { to_draw: s, color: c }
    }
}

pub struct Widget {
    name: String,
    position: Point,
    dimensions: Point,
    pub elements: RwLock<Vec<WidgetElement>>, //to be drawn IN ORDER
    border: Option<BoxType>,

    //Observer Pattern Fields
    observer_id: usize,
    user_input: Arc<UserInput>,
    selection: RwLock<Option<u8>>,

    //Command Pattern Fields
    cmd_queue: CommandQueue,
}

impl Widget {
    pub fn new<T: ToString>(name: T,
                            position: Point,
                            dimensions: Point,
                            user_input: &Arc<UserInput>) -> Self {

        Widget {
            name: name.to_string(),
            position,
            dimensions,
            elements: RwLock::new(Vec::new()),
            border: None,
            observer_id: user_input.generate_id(),
            user_input: user_input.clone(),
            selection: RwLock::new(None),
            cmd_queue: CommandQueue::new(),
        }
    }

    // --- BUILDER PATTERN --- (kinda not a true builder since each fn doesn't return Self)
    pub fn with_border(&mut self, box_type: BoxType) {
        self.border = Some(box_type);
    }

    pub fn with<T: ToString>(&mut self, text: T) { //defaults color to WHITE
        if let Ok(mut elements) = self.elements.write() {
            elements.push(
                WidgetElement {
                    to_draw: text.to_string(),
                    color: RGB::named(WHITE),
                }
            );
        }
    }

    pub fn with_color<T: ToString>(&mut self, text: T, color: RGB) { //no default color, it must be specified
        if let Ok(mut elements) = self.elements.write() {
            elements.push(
                WidgetElement {
                    to_draw: text.to_string(),
                    color,
                }
            );
        }
    }

    pub fn with_these(&mut self, these: &mut Vec<WidgetElement>) {
        if let Ok(mut elements) = self.elements.write() {
            elements.append(these);
        }
    }

    pub fn build(self) -> usize { 
        let id: usize = self.observer_id;

        let arc_widget = Arc::new(self);
        arc_widget.user_input.add_observer(&arc_widget);
        super::widget_storage::add(arc_widget);

        return id;
    }
    // --- END BUILDER PATTERN ---

    //Mutates passed-in DrawBatch, which is drawn only once at the end,
    //in gui::tick().
    pub fn draw(&self, ctx: &mut BTerm, draw_batch: &mut DrawBatch) {
        
        let (x, y, w, h) = (self.position.x,
                            self.position.y,
                            self.dimensions.x,
                            self.dimensions.y);
        let (ctx_w, ctx_h) = ctx.get_char_size();

        //if any part of widget is out of window bounds, return
        if x < 0 || y < 0 || w < 0 || h < 0 { return };
        if x > ctx_w as i32 || y > ctx_h as i32 { return };
        if w > (ctx_w as i32 - x) || h > (ctx_h as i32 - y) { return };

        let mut textblock = TextBlock::new(x + 1, y + 1, w - 1, h - 2);
        let mut textbuilder = TextBuilder::empty();

        if let Ok(elements) = self.elements.read() {
            let mut idx = 0;
            for element in elements.iter() {
                let mut color: RGB = element.color;
                if let Ok(selection) = self.selection.read() {
                    if let Some(focus_idx) = *selection {
                        if focus_idx == idx { color = RGB::named(MAGENTA); };
                    }
                }
                textbuilder.fg(color);
                textbuilder.append(&element.to_draw);
                textbuilder.ln();
                idx += 1;
            }
        }

        textblock.print(&textbuilder);
        textblock.render_to_draw_batch(draw_batch);
        draw_batch.draw_hollow_box(Rect { x1: x, x2: x + w - 1,
                                          y1: y, y2: y + h - 1,
                                   },
                                   ColorPair {
                                       fg: RGBA::named(WHITE),
                                       bg: RGBA::named((0,0,0)),
                                   });
        draw_batch.submit(1000).expect("Batch error in Widget.draw()");
    }

    // Applies delta then clamps/wraps self.selection based on current state.
    fn change_selection (&self, delta: i8) {
        let max;
        if let Ok(elements) = self.elements.read() {
            max = elements.len() - 1;
        } else { panic!("Mutex poisoned in gui::widget::widget_builder.rs"); }

        if let Ok(mut sel_guard) = self.selection.write() {
            if let Some(selection) = *sel_guard {
                let new_selection = selection as i8 + delta;
                if new_selection < 0 { *sel_guard = Some(max as u8); return };
                if new_selection as usize > max { *sel_guard = Some(0); return };
                *sel_guard = Some(new_selection as u8);
            } else { //selection is None
                *sel_guard = Some(0);
            }
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
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
                    InputEvent::ENTER => { Some(Command::Select) },
                    _ => { None },
                };

                if let Some(cmd) = cmd_option {
                    self.send(cmd);
                    self.gui_process();
                }
            }
        }
    }

    //set-up for when this obj becomes what UserInput controls
    fn become_focus(&self) {
        if let Ok(mut sel_guard) = self.selection.write() {
            *sel_guard = Some(0);
            return
        }
        panic!("Mutex was poisoned! (gui::widget::widget_builder::become_focus()");
    }

    fn name(&self) -> &str { &self.name }
}

impl Commandable for Widget {
    fn send(&self, cmd: Command) {
        self.cmd_queue.push(cmd);
    }

    fn gui_process(&self) {
        for cmd in &self.cmd_queue.iter() {
            match cmd {
                Command::Move{dir} => {
                    match dir {
                        Dir::UP => { self.change_selection(-1); },
                        Dir::DOWN => { self.change_selection(1); },
                        Dir::LEFT | Dir::RIGHT => {},
                    }
                },
                Command::Select => {
                    let selection;
                    if let Ok(sel_guard) = self.selection.read() {
                        selection = *sel_guard;
                    } else { panic!("Mutex poisoned! (gui::widget_builder::process())"); }
                        
                    self.user_input.set_focus_selection(selection);
                }
                _ => {},
            }
        }

        self.cmd_queue.clear();
    }

    fn ecs_process(&self, _ecs: &mut World, runstate: RunState) -> RunState {
        //donothing
        runstate
    }
}
