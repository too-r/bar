/*Credit for most of this code goes to UndeadLeech (https://github.com/chrisduerr/bar-helpers)
 This is very rough and pretty crappy code, but it works for me as a lemonbar thingy. You will need to change the path of the config file in config.rs if you want to use a different location for the TOML config file.
 An example config is included in my dotfiles repository, too-r/dotfiles.
*/
extern crate i3ipc;
extern crate time;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate serde;
extern crate regex;
extern crate libudev;

mod config;

use i3ipc::I3Connection;
use time::Duration;
use regex::Regex;
use libudev::{Context, Monitor};
use config::Config;
use std::process::{Command, Stdio};
use std::io::prelude::*;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::thread;

#[cfg(test)]
mod tests {
    use config::parse_config;
    use arch_updates;
    
    #[test]
    fn test_updates() -> () {
        println!("{}", arch_updates(&parse_config()))
    }
}

//A screen as seen by Xorg.
struct Screen {
    name: String,
    xres: String,
    xoffset: String,
}

//The lemonbar struct, describing the lemonbar process as a child of the main process, and the
//screen as an X screen.
struct Lemonbar {
    bar: std::process::Child,
    screen: Screen,
    pow_block: String,
}

//Function that we use to format output and write it
fn add_reset(input: &str) -> String {
    format!("{}%{{B-}}%{{F-}}%{{T-}}", input)
}

//Returns the number of pacman updates available. Planned: Add pacaur.
fn arch_updates(config: &Config) -> String {
    let pacman = Command::new("bash")
        .args(&["-c", "checkupdates | wc -l"])
        .output();
    
    //Make sure the command output worked.
    let pacman_out = pacman.unwrap(); //Output only returns an "Ok" type and no error, so we just unwrap.

    let pacaur = Command::new("bash")
        .args(&["-c", "pacaur -k | wc -l"])
        .output();

    let pacaur_out = pacaur.unwrap();

    let pacman_result = String::from_utf8(pacman_out.stdout).unwrap();
    let pacaur_result = String::from_utf8(pacaur_out.stdout).unwrap();

    let package_count = (pacman_result.parse::<i32>().unwrap() + pacaur_result.parse::<i32>().unwrap()).to_string(); //Parse each thing to an i32 and then back to a string again, woo.
    add_reset(&format!("{}%{{B{}}}%{{F{}}}%{}{}{}",
                       package_count,
                       config.colours.bg_sec,
                       config.colours.fg_col,
                       config.placeholders.updates,
                       config.general.update_icon,
                       config.placeholders.updates))
}

fn now_playing(config: &Config) -> String {
    let meta_artist = Command::new("bash")
        .args(&["-c", "playerctl -p spotify metadata xesam:artist"])
        .output()
        .unwrap()
        .stdout;

    let meta_title = Command::new("bash")
        .args(&["-c", "playerctl -p spotify metadata xesam:title"])
        .output()
        .unwrap()
        .stdout;
    
    let meta_artist_str = String::from_utf8(meta_artist).unwrap();
    let meta_title_str = String::from_utf8(meta_title).unwrap();

    let nowplaying = format!(" {} - {}", meta_artist_str, meta_title_str);

    add_reset(&format!("%{{B{}}}%{{F{}}}{}{}{}",
                       config.colours.bg_col,
                       config.colours.fg_col,
                       config.placeholders.music,
                       nowplaying,
                       config.placeholders.music))
}

//Information about the current set of workspaces.
fn get_ws(screen: &str,
          config: &Config,
          display_count: &i32,
          workspaces: &[i3ipc::reply::Workspace]) -> String {
    let mut result_str = String::new();

    for (i, icon) in config.general.ws_icons.chars().enumerate() {
        let mut ws_index = None;
        for (x, workspace) in workspaces.iter().enumerate() {
            if &workspace.output == screen {
                let normed_ws_num = (workspace.num - 1) / display_count;
                if normed_ws_num == i as i32 {
                    ws_index = Some(x);
                }
            }
        }

        let (col_prim, col_sec) = match ws_index {
            None => (&config.colours.bg_col, &config.colours.bg_sec),
            Some(i) => {
                if workspaces[i].visible {
                    (&config.colours.bg_sec, &config.colours.fg_col) //Has open windows, but is not focused.
                } else if workspaces[i].urgent {
                    (&config.colours.bg_col, &config.colours.hl_col) //Urgent workspace.
                } else {
                    (&config.colours.bg_col, &config.colours.fg_sec) //Focused workspace.
                }
            }
        };

        let ws_script = format!("{} {}", config.executables.workspace, i + 1);
        result_str = format!("{}%{{B{}}}%{{F{}}}%{{A:{}:}}{}{}{}%{{A}}",
                             result_str,
                             col_prim,
                             col_sec,
                             ws_script,
                             config.placeholders.workspace,
                             icon,
                             config.placeholders.workspace);
    }

    add_reset(&result_str)
}

//What is the time my dude?
fn get_date(config: &Config) -> String {
    let current = time::now();
    
    //Check if the current time can be formatted correctly as specified. Planned: Make time
    //formatting a config thing.
    let current_time_clock = match current.strftime("%H:%M") {
        Ok(fmt) => fmt,
        Err(_) => return String::new(),
    };

    add_reset(&format!("%{{B{}}}%{{F{}}}{}{}{}",
                       config.colours.bg_sec,
                       config.colours.fg_col,
                       config.placeholders.clock,
                       current_time_clock,
                       config.placeholders.clock))
}

fn get_vol(screen: &str, config: &Config) -> String {
    let cmd_out = Command::new("bash")
        .args(&["-c",
                "pactl list sinks | grep '^[[:space:]]Volume:' | head -n 1 | tail -n 1 | sed -e \
                 's,.* \\([0-9][0-9]*\\)%.*,\\1,'"])
        .output();

    match cmd_out {
        Ok(out) => {
            let vol_script = format!("{} {} &", config.executables.volume, screen);
            let vol = String::from_utf8_lossy(&out.stdout);
            let vol = vol.trim();

            add_reset(&format!("%{{B{}}}%{{F{}}}%{{A:{}:}}{} {}{}%{{A}}",
                               config.colours.bg_sec,
                               config.colours.fg_col,
                               vol_script,
                               config.placeholders.volume,
                               vol,
                               config.placeholders.volume))
        }
        Err(_) => String::new(),
    }
}

//This is just a placeholder function, we don't use power atm.
fn get_pow(config: &Config) -> String {
    add_reset(&format!("%{{B{}}}%{{F{}}}{}{}{}",
                       config.colours.bg_sec,
                       config.colours.fg_col,
                       config.placeholders.power,
                       config.general.power_icon,
                       config.placeholders.power))
}

//This function returns an array of screens. We make it a Vec so if more screens are plugged in, we
//can account for that.
fn get_screens() -> Vec<Screen> {
    let mut screens = Vec::new();

    let xrandr_out = match Command::new("xrandr").output() {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    let xrandr_str = String::from_utf8_lossy(&xrandr_out.stdout);
    let screen_re = Regex::new("([a-zA-Z0-9-]*) connected .*?([0-9]*)x[^+]*\\+([0-9]*)").unwrap();
    
    //Iterate through all the capture groups in xrandr_str.
    for caps in screen_re.captures_iter(&xrandr_str) {
        //For each screen found that matches the regex, push a Screen instance to the Vec.
        screens.push(Screen {
            name: caps.get(1).unwrap().to_owned().as_str().to_string(), //Messy as hell but it works.
            xres: caps.get(2).unwrap().to_owned().as_str().to_string(),
            xoffset: caps.get(3).unwrap().to_owned().as_str().to_string(),
        });
    }

    screens
}

//Return a growable array of workspaces
fn i3_get_ws(i3con: &mut I3Connection) -> Vec<i3ipc::reply::Workspace> {
    match i3con.get_workspaces() {
        Ok(gw) => gw.workspaces, //Workspace Vec was returned correctly
        Err(_) => {
            *i3con = match I3Connection::connect() {
                //Make a new i3 connection.
                Ok(i3c) => i3c,
                Err(_) => return Vec::new(),
            };
            match i3con.get_workspaces() {
                Ok(gw) => gw.workspaces, //try to get the workspaces again if the connection to i3 was reestablished
                Err(_) => Vec::new(),
            }
        }
    }
}

fn main() {
    loop {
        let screens = get_screens();
        let display_count = screens.len() as i32;

        let mut config = config::parse_config();

        let mut lemonbars = Vec::new(); //Vector to contain all the bars we start based on screens.

        let mut i3con = I3Connection::connect().unwrap();
        
        //For each available screen, create a new lemonbar.
        for screen in screens {
            let rect = format!("{}x{}+{}+0",
                               screen.xres,
                               config.general.height,
                               screen.xoffset);
            let mut lemonbar = Command::new("lemonbar")
                .args(&["-g",
                      &rect[..],
                      "-F",
                      &config.colours.fg_col[..],
                      "-B",
                      &config.colours.bg_col[..],
                      "-f",
                      &config.general.font[..],
                      "-f",
                      &config.general.icon_font[..],
                      "-u",
                      &(&config.general.underline_height).to_string(),
                      "-o",
                      &(&config.general.underline_height / -2).to_string()])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            
            //Thread that controls executing lemonbar output.
            let stdout = lemonbar.stdout.take().unwrap();
            thread::spawn(move || unsafe {
                    let _ = Command::new("sh")
                        .stdin(Stdio::from_raw_fd(stdout.into_raw_fd()))
                        .spawn();
            });
            
            let pow = get_pow(&config);
            //Collect all currently running bars so that we can use them later.
            let lemonstruct = Lemonbar {
                bar: lemonbar,
                screen: screen,
                pow_block: pow,
            };
            lemonbars.push(lemonstruct);
        }

        //Setup a listener to do stuff if monitors are added/removed by Xorg.
        let context = Context::new().unwrap();
        let mut monitor = Monitor::new(&context).unwrap();
        monitor.match_subsystem("drm").unwrap();
        let mut socket = monitor.listen().unwrap();


        let mut curr_time = time::now();
        loop {
            let elapsed = time::now() - curr_time;
            if elapsed >= Duration::seconds(3) {
                curr_time = time::now();
                config = config::parse_config();

                //If there is a monitor event, restart the bar.
                if socket.receive_event().is_some() {
                    for lemonbar in &mut lemonbars {
                        let _ = lemonbar.bar.kill();
                    }
                    break;
                }
            }

            let workspaces = i3_get_ws(&mut i3con);
            let date_block = get_date(&config);

            for lemonbar in &mut lemonbars {
                let stdin = lemonbar.bar.stdin.as_mut().unwrap();

                let ws_block = get_ws(&lemonbar.screen.name, &config, &display_count, &workspaces);
                let vol_block = get_vol(&lemonbar.screen.name, &config);
                let music = now_playing(&config);

                let bar_string = format!("%{{O10000}}%{{U{}+u}}%{{l}}{}{}{}%{{c}}{}%{{r}}{}{}{}\n",
                                         config.colours.hl_col,
                                         lemonbar.pow_block,
                                         config.placeholders.general,
                                         ws_block,
                                         date_block,
                                         config.placeholders.general,
                                         music,
                                         vol_block);

                let _ = stdin.write((&bar_string[..]).as_bytes());
            }
            thread::sleep(Duration::milliseconds(100).to_std().unwrap());
            continue; //Rerun loop, mainly for the purpose of music.
        }
    }
}
