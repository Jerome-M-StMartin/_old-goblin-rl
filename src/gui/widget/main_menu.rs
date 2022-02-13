use super::widget::*;

fn init(user_input: Arc<UserInput>) -> Widget {
    let main_menu_widget: Widget = Widget::new("MainMenu",
                                         Point { x: 14, y: 35 },
                                         Point { x: 11, y: 5 },
                                         user_input,
                                        )
    .with("New Game")
    .with("Load Game")
    .with("Quit Game")
    .build();
    main_menu_widget
}