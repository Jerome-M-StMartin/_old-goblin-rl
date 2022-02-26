use bracket_lib::prelude::*;
use super::{LogFragment, append_entry};

//BUILDER PATTERN

pub struct Logger {
    current_color : RGB,
    fragments : Vec<LogFragment>
}

impl Logger {
    pub fn new() -> Self {
        Logger{
            current_color : RGB::named(WHITE),
            fragments : Vec::new()
        }
    }

    pub fn color(&mut self, color: (u8, u8, u8)) {
        self.current_color = RGB::named(color);
        //self
    }

    pub fn append<T: ToString>(&mut self, text : T) {
        self.fragments.push(
            LogFragment{
                color : self.current_color,
                text : text.to_string()
            }
        );
        //self
    }

    pub fn log(self) {
        append_entry(self.fragments)
    }
}
