use crate::util::{
    event::{Event, Events},
    StatefulList,
};

use snafu::{ErrorCompat, ResultExt, Snafu};
use std::env;
use std::io;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, Text};
use tui::Terminal;

mod bat;
mod rg;
mod util;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("rg failed: {}", source))]
    RgError { source: rg::Error },
    #[snafu(display("generating terminal: {}", source))]
    TerminalError { source: std::io::Error },
    #[snafu(display("receiving input: {}", source))]
    InputError { source: std::sync::mpsc::RecvError },
}

type Result<T, E = Error> = std::result::Result<T, E>;

enum Mode {
    Files,
    Preview,
}

struct App {
    items: StatefulList<rg::Match>,
    mode: Mode,
    offset: u16,

    last_file_name: Option<String>,
    last_file_contents: Option<String>,
}

impl App {
    fn new(items: Vec<rg::Match>) -> App {
        let offset = items[0].line_number - 1;
        let mut app = App {
            items: StatefulList::with_items(items),
            mode: Mode::Files,
            offset: offset,

            // cache files
            last_file_name: None,
            last_file_contents: None,
        };
        app.cache_preview();
        app
    }

    fn change_mode(&mut self, m: Mode) {
        self.mode = m;
    }

    fn next_file(&mut self) {
        self.items.next();
        self.cache_preview();
    }

    fn prev_file(&mut self) {
        self.items.previous();
        self.cache_preview();
    }

    fn cache_preview(&mut self) {
        let selected = self.items.state.selected().expect("getting selected");
        let result = &self.items.items[selected];

        if let Some(cached_file) = &self.last_file_name {
            if cached_file.as_str() == result.file.as_str() {
                return;
            }
        }

        let contents = bat::display_file(&result.file, result.line_number);
        self.last_file_name = Some(result.file.clone());
        self.last_file_contents = Some(String::from(contents));
        self.offset = result.line_number - 1;
    }

    fn inc_offset(&mut self) {
        self.offset += 1;
    }

    fn dec_offset(&mut self) {
        self.offset -= 1;
    }

    fn get_offset(&self) -> u16 {
        self.offset
    }

    fn get_file_text(&self) -> &String {
        self.last_file_contents
            .as_ref()
            .expect("getting file text from preview")
    }
}

fn run_app() -> Result<()> {
    let results = rg::run_rg(env::args().skip(1).collect::<Vec<_>>()).context(RgError {})?;
    if results.is_empty() {
        println!("no results found!");
        std::process::exit(1);
    }

    let stdout = io::stdout().into_raw_mode().context(TerminalError {})?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context(TerminalError {})?;
    terminal.hide_cursor().context(TerminalError {})?;

    let mut app = App::new(results);

    let events = Events::new();

    loop {
        terminal
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Min(50), Constraint::Percentage(80)].as_ref())
                    .split(f.size());

                let items = app.items.items.iter().map(|i| Text::raw(&i.file));
                let items = List::new(items)
                    .block(Block::default().title("Results").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().modifier(Modifier::BOLD))
                    .highlight_symbol(">");
                f.render_stateful_widget(items, chunks[0], &mut app.items.state);

                let prev_vec = vec![Text::raw(app.get_file_text())];
                let paragraph = Paragraph::new(prev_vec.iter())
                    .block(Block::default().title("Preview").borders(Borders::ALL))
                    .scroll(app.get_offset())
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, chunks[1]);
            })
            .context(TerminalError {})?;

        match events.next().context(InputError {})? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('j') => match app.mode {
                    Mode::Files => {
                        app.next_file();
                    }
                    Mode::Preview => {
                        app.inc_offset();
                    }
                },
                Key::Char('k') => match app.mode {
                    Mode::Files => {
                        app.prev_file();
                    }
                    Mode::Preview => {
                        app.dec_offset();
                    }
                },
                Key::Char('l') => {
                    app.change_mode(Mode::Preview);
                }
                Key::Char('h') => {
                    app.change_mode(Mode::Files);
                }
                _ => {}
            },
        }
    }

    Ok(())
}

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                println!("{}", backtrace);
            }
            1
        }
    });
}
