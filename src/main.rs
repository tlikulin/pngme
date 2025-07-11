#![allow(dead_code)]
mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use clap::Parser;

use args::*;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        PngMeArgs::Encode(encode_args) => commands::encode(encode_args),
        PngMeArgs::Decode(decode_args) => commands::decode(decode_args),
        PngMeArgs::Remove(remove_args) => commands::remove(remove_args),
        PngMeArgs::Print(print_args) => commands::print(print_args),
        PngMeArgs::ScanDir(scan_dir_args) => commands::scan_dir(scan_dir_args),
    }
}
