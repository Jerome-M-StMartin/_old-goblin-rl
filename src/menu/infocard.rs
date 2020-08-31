use rltk::{RGB, Rltk};
use specs::prelude::*;
use std::cmp::max;
use super::{Info, MenuOption};

struct InfoBox {
    pub origin: (u32, u32), //top left corner coordinates
    pub height: u32,
    pub width: u32,
}

impl InfoBox {
    pub fn new_main(screen_scale: (f32, i32, i32)) -> InfoBox { //(scale, center_x, center_y)
        let init_origin: (u32, u32) = (0 as u32, 0 as u32);
        let w: u32 = screen_scale.1 as u32 - (init_origin.0 + 1);
        let h: u32 = (screen_scale.2 as u32 * 2) - (init_origin.1 * 2) - 8;
        InfoBox {
            origin: init_origin,
            height: h,
            width: w,
        }
    }

    pub fn new(origin: (u32, u32), width: u32, height: u32) -> InfoBox {
        InfoBox {
            origin,
            width,
            height,
        }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        ctx.draw_box(self.origin.0, self.origin.1, self.width, self.height,
            RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    }

    pub fn draw_color(&self, ctx: &mut Rltk, color: (u8, u8, u8)) {
        ctx.draw_box(self.origin.0, self.origin.1, self.width, self.height,
            RGB::named(color), RGB::named(rltk::BLACK));
    }
}

pub fn show_infocard(ecs: &World, ctx: &mut Rltk, ent: Entity) {

    let info_storage = ecs.read_storage::<Info>();
    if let Some(ent_info) = &info_storage.get(ent) {

        let main_box = InfoBox::new_main(ctx.get_scale());
        let action_box = InfoBox::new((main_box.origin.0 + 1 as u32,
                                      main_box.origin.1 + 1 as u32),
                                      (main_box.width / 2) - 1,
                                      main_box.height / 4);
        let stat_box = InfoBox::new( ((action_box.origin.0 + action_box.width + 1) as u32,
                                      action_box.origin.1),
                                      action_box.width,
                                      action_box.height);
        let desc_box = InfoBox::new( (main_box.origin.0 + 1 as u32,
                                      action_box.origin.1 + action_box.height as u32 + 1),
                                      main_box.width - 2,
                                      main_box.height / 3);
        let lore_box = InfoBox::new( (main_box.origin.0 + 1 as u32,
                                      desc_box.origin.1 + desc_box.height as u32 + 1),
                                      main_box.width - 2,
                                      main_box.height - (action_box.height + desc_box.height + 4));

        main_box.draw_color(ctx, rltk::GREEN);
        action_box.draw(ctx);
        stat_box.draw(ctx);
        desc_box.draw(ctx);
        lore_box.draw(ctx);
        draw_name(ctx, &ent_info.name, &main_box);
        fill_action_box(ctx, &ent_info.actions, &action_box);
        fill_stat_box(ctx, &stat_box);
        fill_description_box(ctx, &ent_info.desc, &desc_box);
        fill_lore_box(ctx, &lore_box);
    }

}

fn draw_name(ctx: &mut Rltk, name: &String, main_box: &InfoBox) {
    let center_x = main_box.origin.0 + (main_box.width / 2);
    ctx.print_color(center_x - (name.len() / 2) as u32, main_box.origin.1,
                    RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), name);
}

//fn fill_action_box(ctx: &mut Rltk, actions: &Vec<(Action, String)>, action_box: &InfoBox) {
fn fill_action_box(ctx: &mut Rltk, actions: &Vec<(MenuOption, String)>, action_box: &InfoBox) {
    
    ctx.print(action_box.origin.0 + 1, action_box.origin.0, "Actions");
    
    let mut print_x = action_box.origin.0 + 1;
    let mut print_y = action_box.origin.1 + 1;
    let mut longest_action_name: usize = 0;

    //print actions into action_box
    for a in actions.iter() {
        ctx.print_color(print_x, print_y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &a.1);
        print_y += 1;
        longest_action_name = max(longest_action_name, a.1.len());
        if print_y >= action_box.origin.1 + action_box.height - 1 { //start new column
            print_y = action_box.origin.1 + 1;
            print_x += longest_action_name as u32 + 2;
            longest_action_name = 0;
        }
    }   
}

fn fill_stat_box(ctx: &mut Rltk, stat_box: &InfoBox) {
    ctx.print(stat_box.origin.0 + 1, stat_box.origin.1, "Stats");
}

fn fill_description_box(ctx: &mut Rltk, description: &String, desc_box: &InfoBox) {

    ctx.print(desc_box.origin.0 + 1, desc_box.origin.1, "Description");
    
    let desc: String = description.clone();
    let desc_len = desc.len();
    let max_line_len: usize = desc_box.width as usize - 2;
    
    let mut line_y: u32 = desc_box.origin.1 + 1;
    let mut curr_total_len: usize = 0;
    let mut curr_line_len: usize = max_line_len;
    let mut curr_line: &str = &desc[0..max_line_len];
   
    loop {
       
        //eliminate end-of-line blankspace then print
        let line_vec: Vec<char> = curr_line.chars().collect();
        while line_vec[curr_line_len - 1] != ' ' {
            curr_line_len -= 1;
        }
        curr_line = &curr_line[0..curr_line_len];
        ctx.print(desc_box.origin.0 + 1, line_y, &curr_line); // <- PRINT

        //set up for next line ----------------------------|
        line_y += 1;
        curr_total_len += curr_line_len;
        
        if (&curr_total_len + &max_line_len) < (desc_len) {
            let start = curr_total_len;
            let end = curr_total_len + max_line_len;
            curr_line = &desc[start..end];
            curr_line_len = max_line_len; //---------------|

        } else { //at last line
            let start = curr_total_len;
            curr_line = &desc[start..];
            ctx.print(desc_box.origin.0 + 1, line_y, &curr_line); // <- PRINT
            break;
        }
    }
}

fn fill_lore_box(ctx: &mut Rltk, lore_box: &InfoBox) {
     ctx.print(lore_box.origin.0 + 1, lore_box.origin.1, "Lore");   
}
