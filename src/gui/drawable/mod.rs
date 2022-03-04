use std::any::Any;

use bracket_lib::prelude::{BTerm, Point};

use super::look_n_feel::Dir;

pub mod scrollbar;
pub mod subwindow;

pub trait Drawable {
    fn draw(&self, ctx: &mut BTerm);
    fn move_to(&self, pos: Point);
    fn orth_move(&self, direction: Dir);
    fn as_any(&self) -> &dyn Any;
}
