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

fn run_app() -> Result<()> {
    let results = rg::run_rg(env::args().skip(1).collect::<Vec<_>>()).context(RgError {})?;

    let bat_text = bat::display_file(&results[0].file, results[0].line_number);

    let stdout = io::stdout().into_raw_mode().context(TerminalError {})?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context(TerminalError {})?;
    terminal.hide_cursor().context(TerminalError {})?;

    let events = Events::new();
    let bet_vec = vec![Text::raw(&bat_text)];

    let mut items_list = StatefulList::with_items(results);

    loop {
        terminal
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Min(50), Constraint::Percentage(80)].as_ref())
                    .split(f.size());

                let items = items_list.items.iter().map(|i| Text::raw(&i.file));
                let items = List::new(items)
                    .block(Block::default().title("Results").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().modifier(Modifier::BOLD))
                    .highlight_symbol(">");
                f.render_stateful_widget(items, chunks[0], &mut items_list.state);

                let paragraph = Paragraph::new(bet_vec.iter())
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
                    items_list.next();
                }
                Key::Char('k') => {
                    items_list.previous();
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
