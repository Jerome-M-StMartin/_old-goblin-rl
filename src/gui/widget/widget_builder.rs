use std::sync::{Arc, RwLock};

use bracket_lib::prelude::*;
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

pub struct WidgetElement {
    to_draw: String,
    color: RGB,
}
pub struct Widget {
    name: String,
    position: Point,
    dimensions: Point,
    elements: Vec<WidgetElement>, //to be drawn IN ORDER
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
            elements: Vec::new(),
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
        self.elements.push(
            WidgetElement {
                to_draw: text.to_string(),
                color: RGB::named(WHITE),
            }
        );
    }

    pub fn with_color<T: ToString>(&mut self, text: T, color: RGB) { //no default color, it must be specified
        self.elements.push(
            WidgetElement {
                to_draw: text.to_string(),
                color,
            }
        );
    }

    pub fn build(self) { 
        let arc_widget = Arc::new(self);
        arc_widget.user_input.add_observer(&arc_widget);
        super::widget_storage::add(arc_widget);
    }
    // --- END BUILDER PATTERN ---

    pub fn draw(&self, ctx: &mut BTerm) {
        /*TODO:
         * Change to accept textbuilder (and maybe other structs) as argument,
         * such that all widgets can be drawn to a single buffer which is drawn
         * to the context only once per tick, instead of multiple draws for the
         * many widgets.
         */
        let (x, y, w, h) = (self.position.x,
                            self.position.y,
                            self.dimensions.x,
                            self.dimensions.y);
        let (ctx_w, ctx_h) = ctx.get_char_size();
/*
        //return if any part of widget is out of window bounds
        if x < 0 || y < 0 || w < 0 || h < 0 { return };
        if x > ctx_w as i32 || y > ctx_h as i32 { return };
        if w > (ctx_w as i32 - x) || h > (ctx_h as i32 - y) { return };
*/
        let mut draw_batch = DrawBatch::new();
        draw_batch.cls();

        //println!("x: {}, y: {},\nw: {}, h: {},\nctx_w: {}, ctx_h: {}", x, y, w - 2, h - 2, ctx_w, ctx_h);

        let mut textblock = TextBlock::new(0, 0, w - 1, h - 2);
        //let mut textblock = TextBlock::new(x + 1, y + 1, w - 2, h - 2);
        let mut textbuilder = TextBuilder::empty();
        //println!("Calling: .draw() on {}", self.name()); //<------------------------------------------rm

        let mut idx = 0;
        for element in self.elements.iter() {
            let mut color: RGB = RGB::named(WHITE);
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

        textbuilder.reset(); //unnecessary until I pass-by-&mut the textbuilder to this fn
        //TODO: draw border
        textblock.print(&textbuilder);
        textblock.render_to_draw_batch(&mut draw_batch);
        draw_batch.submit(0).expect("Batch error in Widget.draw()");
        render_draw_buffer(ctx).expect("Render error in Widget.draw()");
    }

    // Applies delta then clamps/wraps self.selection based on current state.
    fn change_selection (&self, delta: i8) {
        let max = self.elements.len() - 1;
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
