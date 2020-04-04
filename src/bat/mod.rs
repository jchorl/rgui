use bat::{
    config::{Config, InputFile, StyleComponent, StyleComponents, LineRange, LineRanges, HighlightedLineRanges},
    errors::default_error_handler,
    Controller, HighlightingAssets,
};
use std::ffi::OsStr;

pub fn display_file(filename: &String, line_number: i64) -> String {
    let mut output = Vec::new();

    let config = Config {
        colored_output: false,
        style_components: StyleComponents::new(&[
            StyleComponent::Numbers,
        ]),
        highlighted_lines: HighlightedLineRanges(
            LineRanges::from(
                vec!(
                    LineRange::from(&line_number.to_string()).expect("parsing line number"),
                ),
            )
        ),
        term_width: 100,
        files: vec![InputFile::Ordinary(OsStr::new(&filename))],
        ..Default::default()
    };
    let assets = HighlightingAssets::from_binary();

    let controller = Controller::new(&config, &assets);
    controller.run_with_writer(&mut output, default_error_handler);
    String::from_utf8(output).expect("utf8") // TODO make this return a result
}
