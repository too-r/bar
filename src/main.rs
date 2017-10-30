extern crate leechbar as bar;

use leechbar::{BarBuilder, Component, Text, Background, Alignment, Width};

struct MyComponent;

impl Component for MyComponent {
    fn background(&mut self) -> Option<Background> {
        None
    }

    fn text(&mut self) -> Option<Text> {
        Some(Text::new(String::from("Hello, world!")))
    }

    fn alignment(&mut self) -> Aligment {
        Aligment::CENTER
    }

    fn timeout(&mut self) -> Option<Duration> {
        None
    }

    fn event(&mut self) {}
}

fn main() {
    let mut bar = BarBuilder::new().spawn().unwrap();
    bar.add(MyComponent);
    bar.start_event_loop();
}
