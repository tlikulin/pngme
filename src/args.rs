use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "PngMe")]
#[command(version = "1.0.0")]
#[command(about = "Lets your encode and decode hidden messages in PNG files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: PngMeArgs,
}

#[derive(Subcommand)]
pub enum PngMeArgs {
    /// Put a chunk with your message in a PNG file and write it to disk
    Encode(EncodeArgs),
    /// Read the chunk from a PNG file
    Decode(DecodeArgs),
    /// Remove the chunk from a PNG file
    Remove(RemoveArgs),
    /// Print all chunks of a PNG file
    Print(PrintArgs),
    /// Print out all files in directory, similar to print
    ScanDir(ScanDirArgs),
}

#[derive(Args)]
pub struct EncodeArgs {
    /// PNG file to modify
    pub file_path: PathBuf,
    /// Type (name) of chunk to append
    pub chunk_type: String,
    /// Message to encode
    pub message: String,
    /// Output file with message encoded (overwrites file by default)
    pub output_file: Option<PathBuf>,
}

#[derive(Args)]
pub struct DecodeArgs {
    /// PNG file with a message
    pub file_path: PathBuf,
    /// Type (name) of chunk containing the message
    pub chunk_type: String,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// PNG file to modify
    pub file_path: PathBuf,
    /// Type (name) of chunk to remove
    pub chunk_type: String,
}

#[derive(Args)]
pub struct PrintArgs {
    /// Omit blank, invalid UTF-8, and standard chunks
    #[clap(short, long)]
    pub filter: bool,
    /// PNG file to read
    pub file_path: PathBuf,
}

#[derive(Args)]
pub struct ScanDirArgs {
    /// Omit blank, invalid UTF-8, and standard chunks
    #[clap(short, long)]
    pub filter: bool,
    /// Directory with PNG files to print out (defaults to .)
    pub dir: Option<PathBuf>,
}
