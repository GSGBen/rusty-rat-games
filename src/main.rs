use std::{env, io};

mod game01;
mod game02;
#[allow(dead_code)]
mod shared;

fn main() -> io::Result<()> {
    if let Some(game_number) = env::args().nth(1) {
        match game_number.as_str() {
            "01" => {
                let mut app = game01::App::new();
                ratatui::run(|terminal| app.run(terminal))
            }
            "02" => {
                let mut app = game02::App::new();
                ratatui::run(|terminal| app.run(terminal))
            }
            _ => Ok(()),
        }
    } else {
        println!("Available games (pass in as first arg): 01, 02");
        Ok(())
    }
}
