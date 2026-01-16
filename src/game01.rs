// game01: a few bar graphs of different values. The input val oscillates
// randomly. the goal is to get the output val to a certain level. The player
// presses a button to add the current input val to the output val. This also
// uses some of a third val. The challenge is that you need to add a high-enough
// input val each time to max out the output val before the third val runs out.
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
    layout::{Constraint, HorizontalAlignment, Offset, Rect, Size},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Clear, Paragraph, Widget},
};

use crate::shared::{AudioPlayer, round_precise};

// game values that don't need to change
const TITLE: &str = "01";
const DESC_TEXTS: [&str; 3] = [
    "Goal: Achieve Z, before running out of B",
    "Controls: Spacebar",
    "Quit: Q",
];
const WON_TEXT: &str = "Z achieved - you won!";
const LOST_TEXT: &str = "B ran out - game over!";
/// max value of any value shown in the bar graph, in bar graph units.
const BAR_MAX: u64 = 100;
const SPEED: f64 = 2.0;
/// require MAX_PRESSES presses at an a val of MIN_WIN_PRESS_VAL or above
/// to max out z. if MAX_PRESSES was 10 and MIN_WIN_PRESS_VAL was 0.75,
/// you'd need 10 presses at .75 or above to win.
const MAX_PRESSES: f64 = 10.0;
/// require MAX_PRESSES presses at an a val of MIN_WIN_PRESS_VAL or above
/// to max out z. if MAX_PRESSES was 10 and MIN_WIN_PRESS_VAL was 0.75,
/// you'd need 10 presses at .75 or above to win.
const MIN_WIN_PRESS_VAL: f64 = 0.75;

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
    /// input val. 0 to 1, oscillates
    a_val: f64,
    /// third / health / action points val. 0 to 1, starts at 1
    b_val: f64,
    /// output val. 0 to 1, starts at 0
    z_val: f64,
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
            a_val: 0.0,
            b_val: 1.0,
            z_val: 0.0,
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
        self.a_val = new.a_val;
        self.b_val = new.b_val;
        self.z_val = new.z_val;
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
        // -1 to 1
        let perlin_val = self.perlin.get([self.elapsed.as_secs_f64() * SPEED]);
        // 0 to 1
        let perlin_normalised = (perlin_val + 1.0) / 2.0;
        self.a_val = perlin_normalised;

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
                // convert the current val of a to z, use up some b.
                // see MAX_PRESSES for description of calc
                self.z_val += f64::min(self.a_val / MAX_PRESSES * (1.0 / MIN_WIN_PRESS_VAL), 1.0);
                self.b_val = f64::max(self.b_val - 1.0 / MAX_PRESSES, 0.0);
                // avoid accumulating floating point errors
                self.z_val = round_precise(self.z_val, 2);
                self.b_val = round_precise(self.b_val, 2);

                // calculate win/loss conditions
                if self.z_val >= 1.0 {
                    self.game_state = GameState::Won;
                    self.audio.play_sound_3();
                    return;
                } else if self.b_val <= 0.0 {
                    self.game_state = GameState::Lost;
                    self.audio.play_sound_4();
                    return;
                }

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
        Block::bordered()
            .title(TITLE)
            .title_alignment(HorizontalAlignment::Center)
            .render(area, buf);

        // some debug values down the bottom
        Paragraph::new(format!(
            "\
tick time: {:3} ms
FPS:       {:<5.2}\
",
            self.delta_time.as_millis(),
            // as_seconds() is whole seconds only
            1.0 / (self.delta_time.as_millis() as f64 / 1000.0)
        ))
        .left_aligned()
        .render(
            Rect {
                x: 0,
                y: area.height - 2,
                width: area.width,
                height: area.height,
            },
            buf,
        );

        // description up the top. whole thing centered, but text inside
        // left-aligned.
        let mut longest_desc_str_length = 0;
        for t in DESC_TEXTS.iter() {
            longest_desc_str_length = max(longest_desc_str_length, t.len())
        }
        // + 2 for the border
        let desc_centered = area
            .centered_horizontally(Constraint::Length((longest_desc_str_length + 2) as u16))
            .offset(Offset::new(0, 1))
            .resize(Size::new(
                (longest_desc_str_length + 2) as u16,
                DESC_TEXTS.len() as u16 + 2,
            ));
        Paragraph::new(
            DESC_TEXTS
                .into_iter()
                .map(|t| Line::raw(t))
                .collect::<Vec<Line>>(),
        )
        .block(Block::bordered())
        .render(desc_centered, buf);

        // bar graph in the middle.

        // warn when health is low
        let b_style = if self.b_val <= 0.25 {
            Style::default().light_red().on_red()
        } else {
            Style::default()
        };
        // show win or loss status
        let z_style = match self.game_state {
            GameState::Playing => Style::default(),
            GameState::Won => Style::default().green(),
            GameState::Lost => Style::default().red().on_red(),
        };

        let bars = [
            Bar::with_label("A", (self.a_val * BAR_MAX as f64).round() as u64),
            Bar::with_label("B", (self.b_val * BAR_MAX as f64).round() as u64).style(b_style),
            Bar::with_label("Z", (self.z_val * BAR_MAX as f64).round() as u64).style(z_style),
        ];
        let bar_width = 4;
        let chart_centered = area.centered(
            Constraint::Length(bars.len() as u16 * bar_width + 4),
            Constraint::Percentage(50),
        );
        let bar_style = Style::new().fg(Color::LightBlue).bg(Color::Blue);
        BarChart::default()
            .data(BarGroup::new(bars))
            .block(Block::bordered())
            .bar_width(bar_width)
            .blue()
            .max(BAR_MAX)
            .bar_style(bar_style)
            .render(chart_centered, buf);

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
