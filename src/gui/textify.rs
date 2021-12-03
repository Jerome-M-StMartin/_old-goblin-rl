//Jerome M. St.Martin
//jeromemst.martin@gmail.com

/* This trait is for making drawable/printable strings formatted specifically for bracket-lib's
 * BTerm::TextBlock functionality. For turning any ECS component into visible GUI text.
 */

use bracket_terminal::prelude::{TextBlock, TextBuilder};

use super::look_n_feel::ColorOption;

/* Usage:
 * Some System needs to gather all components on some Focus entity that implement Textify,
 * then insert each's TextBlock into some Drawable struct which has .draw() called on it
 * in the .tick() fn of the GUI.
 */

pub trait Textify {
    fn as_textblock(&self) -> TextBlock;
}

use super::super::components::Stats;
impl Textify for Stats {
    fn as_textblock(&self) -> TextBlock {

        let mut hp: String = String::new();
        let mut fp: String = String::new();
        let mut mp: String = String::new();
        let mut lost_hp: String = String::new();
        let mut lost_fp: String = String::new();
        let mut lost_mp: String = String::new();

        for _ in 0..self.hp { hp.push_str("♥"); }
        for _ in 0..(self.max_hp - self.hp) { lost_hp.push_str("♥"); }

        for _ in 0..self.fp { fp.push_str("♥"); }
        for _ in 0..(self.max_fp - self.fp) { lost_fp.push_str("♥"); }

        for _ in 0..self.mp { mp.push_str("♥"); }
        for _ in 0..(self.max_mp - self.mp) { lost_mp.push_str("♥"); }


        let mut tb = TextBlock::new(0, 0, 12, 4);
        let mut buff = TextBuilder::empty();
        buff.ln()
            .fg(ColorOption::CAUTION.value())
            .bg(ColorOption::NONE.value())
            .centered("STATS")
            .fg(ColorOption::DEFAULT.value())
            .ln()
            .append("HP: ")
            .fg(ColorOption::ALERT.value())
            .append(&hp)
            .fg(ColorOption::DEFAULT.value())
            .append(&lost_hp)
            .ln()
            .append("FP: ")
            .fg(ColorOption::ALERT.value())
            .append(&fp)
            .fg(ColorOption::DEFAULT.value())
            .append(&lost_fp)
            .ln()
            .append("MP: ")
            .fg(ColorOption::ALERT.value())
            .append(&mp)
            .fg(ColorOption::DEFAULT.value())
            .append(&lost_mp);

        tb.print(&buff);
        tb
    }
}

/*
.append("┏")
.append("━━━━━━━━━━")
.append("┓")
 */
