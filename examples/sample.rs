use luigi_rs::{self as ui, Button, Panel, Window};

fn main() {
    // Initialize the UI system
    ui::init();

    // Create main window
    let window = Window::new("Rust UI Example", 800, 600, 0).expect("Failed to create window");

    // Create a panel with gray background
    let panel = Panel::new(&window, ui::UI_PANEL_GRAY).expect("Failed to create panel");

    // Create some buttons
    let _button1 = Button::new(&panel, 0, "Hello").expect("Failed to create button");
    let _button2 = Button::new(&panel, 0, "World").expect("Failed to create button");
    let _button3 = Button::new(&panel, 0, "Click me!").expect("Failed to create button");

    // Start the message loop
    ui::message_loop();
}
