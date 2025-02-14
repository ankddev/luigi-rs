use luigi_rs::{self as ui, Button, Label, Panel, Element, Window};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Initialize UI
    ui::init();

    // Create main window and save it
    let window = Window::new("Counter", 200, 150, 0)
        .expect("Failed to create window");

    // Create container panel with white background
    let panel = Panel::new(&window, ui::UI_PANEL_WHITE | ui::UI_PANEL_MEDIUM_SPACING)
        .expect("Failed to create panel");

    // Create label and count to be shared between callbacks
    let label = Rc::new(RefCell::new(Label::new(&panel, 0, &format!("{:>3}", 0)).expect("Failed to create label")));
    let count = Rc::new(RefCell::new(0));

    // Create buttons panel
    let buttons = Panel::new(&panel, ui::UI_PANEL_HORIZONTAL)
        .expect("Failed to create buttons panel");

    // Create and store minus button callback
    let label_clone = label.clone();
    let count_clone = count.clone();
    let minus_callback = Box::new(move || {
        *count_clone.borrow_mut() -= 1;
        let mut label = label_clone.borrow_mut();
        // Format the number into a string with proper width
        label.set_content(&format!("{:>3}", *count_clone.borrow()));
        label.refresh();
    });
    let minus = Button::new(&buttons, 0, "-")
        .expect("Failed to create minus button");
    minus.invoke(minus_callback);

    // Create and store plus button callback
    let plus_callback = Box::new(move || {
        *count.borrow_mut() += 1;
        let mut label = label.borrow_mut();
        // Format the number into a string with proper width
        label.set_content(&format!("{:>3}", *count.borrow()));
        label.refresh();
    });
    let plus = Button::new(&buttons, 0, "+")
        .expect("Failed to create plus button"); 
    plus.invoke(plus_callback);

    // Start the message loop
    ui::message_loop();
}
