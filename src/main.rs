extern crate leechbar;
extern crate chan;
extern crate time;
extern crate env_logger;

use leechbar::{BarBuilder, Bar, Component, Text, Background, Foreground, Alignment, Width, Color};
use std::time::Duration;

mod time_component;
use time_component::Time;

fn main() {
    env_logger::init().unwrap();


    let mut bar = BarBuilder::new()
        .background_color(Color::new(27, 27, 27, 255))
        .foreground_color(Color::new(97, 97, 97, 255))
        .spawn()
        .unwrap();

    //Create time component.
    let time = Time::new(bar.clone());

    bar.add(time);
    bar.start_event_loop();
}
