//Jerome M. St.Martin
//Node Menu Project
//12/18/2020

use bracket_lib::prelude::{BLACK, GREEN, MAGENTA, RED, WHITE, YELLOW};
use serde::{Deserialize, Serialize};

//--- Standardization of Colors ----
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ColorOption {
    FOCUS,
    ALERT,
    CAUTION,
    OK,
    NONE, //black
    DEFAULT,
}

impl ColorOption {
    pub fn value(&self) -> (u8, u8, u8) {
        match *self {
            ColorOption::DEFAULT => WHITE,
            ColorOption::FOCUS => MAGENTA,
            ColorOption::ALERT => RED,
            ColorOption::CAUTION => YELLOW,
            ColorOption::OK => GREEN,
            ColorOption::NONE => BLACK,
        }
    }
}//----------------------------------------

// Orthogonal Direction
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Dir {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}
