use snafu::{ErrorCompat, ResultExt, Snafu};
use std::env;
use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Paragraph, SelectableList, Text};
use tui::layout::{Alignment, Layout, Constraint, Direction};
use tui::style::{Color, Modifier, Style};

mod bat;
mod rg;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("rg failed: {}", source))]
    RgError { source: rg::Error },
    #[snafu(display("generating terminal: {}", source))]
    TerminalError { source: std::io::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn run_app() -> Result<()> {
    let results = rg::run_rg(env::args().skip(1).collect::<Vec<_>>()).context(RgError {})?;
    let mut filenames = Vec::new();
    for r in &results {
        filenames.push(&r.file);
    }

    let mut selected = 0;
    let selected_result = &results[selected];
    let bat_text = bat::display_file(&selected_result.file, selected_result.line_number);

    let stdout = io::stdout().into_raw_mode().context(TerminalError {})?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context(TerminalError {})?;
    terminal.clear().context(TerminalError {})?;
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Min(50),
                    Constraint::Percentage(80),
                ].as_ref()
            )
            .split(f.size());
        SelectableList::default()
            .block(Block::default().title("Results").borders(Borders::ALL))
            .items(&filenames[..])
            .select(Some(selected))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::BOLD))
            .highlight_symbol(">>")
            .render(&mut f, chunks[0]);
        Paragraph::new([Text::raw(bat_text)].iter())
            .block(Block::default().title("Preview").borders(Borders::ALL))
            .alignment(Alignment::Left)
            .render(&mut f, chunks[1]);
    }).context(TerminalError {})?;

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
