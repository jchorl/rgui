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

struct App {
    items: StatefulList<rg::Match>,
}

impl App {
    fn new(items: Vec<rg::Match>) -> App {
        App {
            items: StatefulList::with_items(items),
        }
    }

    fn get_file_text(&self) -> String {
        let selected = self.items.state.selected().unwrap();
        let result = &self.items.items[selected];
        let contents = bat::display_file(&result.file, result.line_number);
        String::from(contents)
    }
}

fn run_app() -> Result<()> {
    let results = rg::run_rg(env::args().skip(1).collect::<Vec<_>>()).context(RgError {})?;

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
                    .alignment(Alignment::Left);
                f.render_widget(paragraph, chunks[1]);
            })
            .context(TerminalError {})?;

        match events.next().context(InputError {})? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('j') => {
                    app.items.next();
                }
                Key::Char('k') => {
                    app.items.previous();
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
