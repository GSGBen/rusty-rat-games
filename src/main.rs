use std::{env, io};

mod game01;
#[allow(dead_code)]
mod shared;

fn main() -> io::Result<()> {
    if let Some(game_number) = env::args().nth(1) {
        match game_number.as_str() {
            "01" => {
                let mut app = game01::App::new();
                ratatui::run(|terminal| app.run(terminal))
            }
            _ => Ok(()),
        }
    } else {
        println!("Available games (pass in as first arg): 01");
        Ok(())
    }
}
