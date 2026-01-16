// game02:
//

use std::{
    cmp::max,
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{
        Constraint, Direction, HorizontalAlignment, Layout, Offset, Rect, Size, VerticalAlignment,
    },
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Clear, Paragraph, Widget},
};

use crate::shared::{AudioPlayer, round_precise};

// game values that don't need to change
const TITLE: &str = "02";
const WON_TEXT: &str = "Z achieved - you won!";
const LOST_TEXT: &str = "B ran out - game over!";

#[derive(Debug)]
enum GameState {
    Playing,
    Won,
    Lost,
}

/// global data that the game needs to adjust
#[derive(Debug)]
pub struct App {
    audio: AudioPlayer,
    exit: bool,
    now: Instant,
    /// tick time
    delta_time: Duration,
    /// time since game start
    elapsed: Duration,
    perlin: Perlin,
    game_state: GameState,
}

impl App {
    /// initialize all data
    pub fn new() -> Self {
        let mut rng = rand::rng();
        App {
            audio: AudioPlayer::new(),
            exit: false,
            now: Instant::now(),
            // avoid divide-by-zero by not initialising to zero
            delta_time: Duration::from_millis(1),
            elapsed: Duration::from_millis(0),
            perlin: noise::Perlin::new(rng.random()),
            game_state: GameState::Playing,
        }
    }

    /// reset just the game variables back to defaults
    fn reset(&mut self) {
        let new = App::new();
        self.exit = new.exit;
        self.now = new.now;
        self.delta_time = new.delta_time;
        self.elapsed = new.elapsed;
        self.perlin = new.perlin;
        self.game_state = new.game_state;
    }

    /// this runs the main outer loop. Setup code can go here before the loop,
    /// and non-game per-tick things can go in the loop to keep tick() clean.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let started_at = Instant::now();

        while !self.exit {
            // update time values
            let new_now = Instant::now();
            self.delta_time = new_now.duration_since(self.now);
            self.elapsed = new_now.duration_since(started_at);
            self.now = new_now;

            // take input
            self.handle_events()?;

            // update per-tick game state
            self.tick()?;

            // draw new game state
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    /// the main gameplay per-tick function.
    fn tick(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_micros(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Char(' ') => self.pressed_space(),
                        KeyCode::Char('y') => self.pressed_y(),
                        KeyCode::Char('n') => self.pressed_n(),
                        _ => {}
                    }
                }
                _ => {}
            };
        }
        Ok(())
    }

    /// this is a very small function to satisfy the borrow checker, and the
    /// difference between &App and &mut App, when called via the closure passed
    /// to terminal.draw().
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn pressed_space(&mut self) {
        match self.game_state {
            GameState::Playing => {
                self.audio.play_sound_2();
            }
            _ => {}
        }
    }

    fn pressed_y(&mut self) {
        match self.game_state {
            GameState::Won => {
                self.reset();
            }
            GameState::Lost => {
                self.reset();
            }
            _ => {}
        }
    }

    fn pressed_n(&mut self) {
        match self.game_state {
            GameState::Won => {
                self.exit();
            }
            GameState::Lost => {
                self.exit();
            }
            _ => {}
        }
    }
}

impl Widget for &App {
    /// main draw call
    fn render(self, area: Rect, buf: &mut Buffer) {
        // main outer border
        Block::default()
            .title(TITLE)
            .title_alignment(HorizontalAlignment::Center)
            .render(area, buf);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(1),
            ])
            .split(area);

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(main_layout[2]);
        Block::bordered().render(main_layout[1], buf);
        Block::bordered().render(bottom_layout[0], buf);
        Block::bordered().render(bottom_layout[1], buf);
        Block::bordered().render(bottom_layout[2], buf);

        // some debug values down the bottom
        Paragraph::new(format!(
            "tick time: {:3} ms, FPS: {:<5.2}",
            self.delta_time.as_millis(),
            // as_seconds() is whole seconds only
            1.0 / (self.delta_time.as_millis() as f64 / 1000.0)
        ))
        .left_aligned()
        .render(
            Rect {
                x: 0,
                y: area.height - 1,
                width: area.width,
                height: area.height,
            },
            buf,
        );

        // win/loss popup
        if !matches!(self.game_state, GameState::Playing) {
            let end_box_centered =
                area.centered(Constraint::Percentage(50), Constraint::Percentage(25));
            let outcome_line = Line::raw(if matches!(self.game_state, GameState::Won) {
                WON_TEXT
            } else {
                LOST_TEXT
            })
            .style(if matches!(self.game_state, GameState::Won) {
                Style::new().green()
            } else {
                Style::new().red()
            })
            .bold();
            Clear::default().render(end_box_centered, buf);
            Paragraph::new(vec![
                Line::default(),
                Line::default(),
                Line::default(),
                outcome_line,
                Line::default(),
                "Play again?".into(),
                "(Y)es (N)o".into(),
            ])
            .centered()
            .block(Block::bordered())
            .render(end_box_centered, buf);
        }
    }
}
