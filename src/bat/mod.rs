use bat::{
    config::{Config, InputFile, StyleComponent, StyleComponents},
    Controller, HighlightingAssets,
};
use std::ffi::OsStr;

pub fn display_file(filename: &String) {
    let config = Config {
        colored_output: true,
        true_color: true,
        style_components: StyleComponents::new(&[
            StyleComponent::Header,
            StyleComponent::Grid,
            StyleComponent::Numbers,
        ]),
        files: vec![InputFile::Ordinary(OsStr::new(filename))],
        ..Default::default()
    };
    let assets = HighlightingAssets::from_binary();

    Controller::new(&config, &assets).run().expect("no errors");
}
