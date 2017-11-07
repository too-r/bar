use leechbar::*;
use time;
use chan;
use std::thread;
use std::time::Duration;

pub struct Time {
    bar: Bar,
    last_content: String,
    last_text: Option<Text>,
}

impl Time {
    pub fn new(bar: Bar) -> Time {
        Time {
            bar: bar,
            last_content: String::new(),
            last_text: None,
        }
    }
}

impl Component for Time {
    fn update(&mut self) -> bool {
        let time = time::now();

        let content = format!("{:02}:{:02}", time.tm_hour, time.tm_min);

        if content != self.last_content {
            self.last_text = if !content.is_empty() {
                self.last_content = content;
                Some(Text::new(&self.bar, &self.last_content, None, None).unwrap())
            } else {
                None
            };

            true
        } else {
            false
        }
    }
    
    fn background(&self) -> Background {
        Background::new()
            .color(Color::new(38, 38, 38, 255))
    }

    fn foreground(&self) -> Foreground {
        if let Some(ref last_text) = self.last_text {
            last_text.clone().into()
        } else {
            Foreground::new()
        }
    }

    fn redraw_timer(&mut self) -> chan::Receiver<()> {
        let (tx, rx) = chan::sync(0);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(15));
            let _ = tx.send(());
        });

        rx
    }

    fn width(&self) -> Width {
        Width::new().fixed(100)
    }

    fn alignment(&self) -> Alignment {
        Alignment::CENTER
    }

}
