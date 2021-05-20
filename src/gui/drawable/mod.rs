/* Jerome M. St.Martin
 * 12/07/2020 - 5/18/2020
 * jeromemst.martin@gmail.com
 */

pub mod scrollbar;
pub mod subwindow;

use bracket_terminal::prelude::{BTerm, Point};
use super::look_n_feel::Dir;

pub trait Drawable {
    fn draw(&self, ctx: &mut BTerm);
    fn move_to(&self, pos: Point);
    fn orth_move(&self, direction: Dir);
}
