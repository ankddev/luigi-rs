<div align="center">

# Luigi-rs

Safe Rust bindings for [Luigi C UI library](https://github.com/nakst/luigi). Note that it supports only Linux and Windows!

[![Crates.io](https://img.shields.io/crates/v/luigi-rs.svg)](https://crates.io/crates/luigi-rs)
[![Downloads](https://img.shields.io/crates/d/luigi-rs.svg)](https://crates.io/crates/luigi-rs)

</div>

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
luigi-rs = "1.0.0"
```
or run this command:
```bash
cargo add luigi-rs
```

## Features

- Safe Rust interface for Luigi UI library
- Zero-cost abstractions over the C API
- Windows and Linux support

## Example

Here's a simple counter application:

```rust
use luigi_rs::{self as ui, Button, Label, Panel, Window};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    ui::init();

    let window = Window::new("Counter", 200, 150, 0).expect("Failed to create window");
    let panel = Panel::new(&window, ui::UI_PANEL_WHITE).expect("Failed to create panel");
    
    let label = Rc::new(RefCell::new(Label::new(&panel, 0, "0").expect("Failed to create label")));
    let count = Rc::new(RefCell::new(0));

    let buttons = Panel::new(&panel, ui::UI_PANEL_HORIZONTAL).expect("Failed to create buttons panel");

    // Create minus button
    let label_clone = label.clone();
    let count_clone = count.clone();
    let minus = Button::new(&buttons, 0, "-").expect("Failed to create minus button");
    minus.invoke(Box::new(move || {
        *count_clone.borrow_mut() -= 1;
        label_clone.borrow_mut().set_content(&count_clone.borrow().to_string());
    }));

    // Create plus button
    let plus = Button::new(&buttons, 0, "+").expect("Failed to create plus button");
    plus.invoke(Box::new(move || {
        *count.borrow_mut() += 1;
        label.borrow_mut().set_content(&count.borrow().to_string());
    }));

    ui::message_loop();
}
```

## Building from Source

1. First, ensure you have the [bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html) installed.

2. On Linux, install required X11 development packages:
```bash
sudo apt-get install libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev
```

3. Clone the repository:
```bash
git clone https://github.com/ankddev/luigi-rs
cd luigi-rs
```

4. Build the project:
```bash
cargo build
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork it (https://github.com/ankddev/luigi-rs/fork)
2. Create your feature branch (`git checkout -b feature/something`)
3. Commit your changes (`git commit -am 'Add something'`)
4. Push to the branch (`git push origin feature/something`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

[ANKDDEV](https://github.com/ankddev)

## Acknowledgments

- [Luigi UI library](https://github.com/nakst/luigi) by [nakst](https://github.com/nakst)
- All contributors to this project
