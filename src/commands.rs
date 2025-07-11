use crate::args::{DecodeArgs, EncodeArgs, PrintArgs, RemoveArgs, ScanDirArgs};
use crate::{chunk::Chunk, chunk_type::ChunkType, png::Png};
use colored::{ColoredString, Colorize};
use std::{env, fs, path::PathBuf};

pub fn encode(encode_args: EncodeArgs) {
    let EncodeArgs {
        file_path,
        chunk_type: chunk_name,
        message,
        output_file,
    } = encode_args;
    let file_name = file_path.display().to_string();
    let data = message.as_bytes().to_owned();
    let output_file = output_file.unwrap_or(file_path.clone());

    let mut png = match read_png(file_path) {
        Some(png) => png,
        None => {
            println!("Not valid PNG format");
            std::process::exit(1);
        }
    };

    let chunk_type = match chunk_name.parse::<ChunkType>() {
        Ok(ct) if ct.is_valid() => ct,
        _ => {
            println!(
                "{chunk_name} is not a valid chunk type, should be 4 letters with 3rd uppercase"
            );
            std::process::exit(1);
        }
    };

    let chunk = Chunk::new(chunk_type, data);

    png.append_chunk(chunk);
    match fs::write(output_file, png.as_bytes()) {
        Ok(()) => println!("Added \"{message}\" to file `{file_name}` as chunk {chunk_name}"),
        Err(e) => println!("Could not save file: {e}"),
    }
}

pub fn decode(decode_args: DecodeArgs) {
    let DecodeArgs {
        file_path,
        chunk_type,
    } = decode_args;
    let file_name = file_path.display().to_string();

    let png = match read_png(file_path) {
        Some(png) => png,
        None => {
            println!("Not valid PNG format");
            std::process::exit(1);
        }
    };

    let output = match png.chunk_by_type(&chunk_type) {
        Some(chunk) => format!("{chunk_type}: {}", get_formatted_chunk_message(chunk)),
        None => format!("Chunk {chunk_type} not found"),
    };

    println!("In {file_name},");
    println!("{output}");
}

pub fn remove(remove_args: RemoveArgs) {
    let RemoveArgs {
        file_path,
        chunk_type,
    } = remove_args;
    let file_name = file_path.display().to_string();

    let mut png = match read_png(file_path.clone()) {
        Some(png) => png,
        None => {
            println!("Not valid PNG format");
            std::process::exit(1);
        }
    };

    let chunk = match png.remove_first_chunk(&chunk_type) {
        Ok(ch) => ch,
        Err(_) => {
            println!("Chunk {chunk_type} not found in `{file_name}`");
            std::process::exit(1);
        }
    };

    match fs::write(file_path, png.as_bytes()) {
        Ok(()) => println!(
            "Chunk {chunk_type} with contents \"{}\" was removed from `{file_name}`",
            get_formatted_chunk_message(&chunk)
        ),
        Err(e) => println!("Could not save file: {e}"),
    }
}

pub fn print(print_args: PrintArgs) {
    let PrintArgs { filter, file_path } = print_args;
    let file_name = file_path.display().to_string();

    let png = match read_png(file_path) {
        Some(png) => png,
        None => {
            println!("Not valid PNG format");
            std::process::exit(1);
        }
    };

    println!("File: `{}`", file_name);
    println!("-------------------------------");
    analyse_png(&png, filter);
}

pub fn scan_dir(scan_dir_args: ScanDirArgs) {
    let ScanDirArgs { filter, dir } = scan_dir_args;
    let dir = dir.unwrap_or(env::current_dir().unwrap());

    let dir_entries = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            println!("Could not read directory: {e}");
            std::process::exit(1);
        }
    };

    for entry in dir_entries {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            let file_name = file_path.display().to_string();
            let Some(png) = read_png_quiet(file_path) else {
                continue;
            };

            println!("File: `{file_name}`");
            println!("-------------------------------");
            analyse_png(&png, filter);
        }
    }
}

fn read_png(file_path: PathBuf) -> Option<Png> {
    let display = file_path.display();

    // check file exists and not a dir
    if !file_path.is_file() {
        println!("`{display}` does not exist or is not a file");
        std::process::exit(1);
    }

    // check extension
    if let Some(ext) = file_path.extension()
        && ext != "png"
    {
        println!(
            "`{display}' has extension `{}`, expected `png`",
            ext.display()
        );
        std::process::exit(1);
    } else if file_path.extension().is_none() {
        println!("'{display}' has no extension, expected `png`");
        std::process::exit(1);
    }

    // read as bytes and build a struct
    let buffer = match fs::read(file_path) {
        Ok(buf) => buf,
        Err(e) => {
            println!("Could not read file: {e}");
            std::process::exit(1);
        }
    };
    let png = Png::try_from(&buffer[..]).ok();

    png
}

fn read_png_quiet(file_path: PathBuf) -> Option<Png> {
    // check file exists and not a dir
    if !file_path.is_file() {
        return None;
    }

    // check extension
    if let Some(ext) = file_path.extension()
        && ext != "png"
    {
        return None;
    } else if file_path.extension().is_none() {
        return None;
    }

    // read as bytes and build a struct
    let buffer = fs::read(file_path).ok()?;
    let png = Png::try_from(&buffer[..]).ok();

    png
}

fn analyse_png(png: &Png, filter: bool) {
    let chunks = png.chunks().iter();

    if filter {
        let chunks = chunks.filter(filtering);
        if chunks.clone().next().is_none() {
            println!("{}", "* All chunks are filtered out *\n".yellow());
        }

        for (ind, chunk) in chunks.enumerate() {
            println!(
                "Chunk #{ind}: {chunk_type}",
                chunk_type = chunk.chunk_type()
            );
            println!(
                "Length {length}, CRC {crc}",
                length = chunk.length(),
                crc = chunk.crc()
            );

            let message = get_formatted_chunk_message(chunk);
            println!("Contains: {message}\n");
        }
    } else {
        for (ind, chunk) in chunks.enumerate() {
            println!(
                "Chunk #{ind}: {chunk_type}",
                chunk_type = chunk.chunk_type()
            );
            println!(
                "Length {length}, CRC {crc}",
                length = chunk.length(),
                crc = chunk.crc()
            );

            let message = get_formatted_chunk_message(chunk);
            println!("Contains: {message}\n");
        }
    }
}

fn filtering(chunk: &&Chunk) -> bool {
    let excluded = [b"IHDR", b"IDAT", b"IEND"];

    chunk.length() != 0
        && chunk.data_as_string().is_ok()
        && !excluded.contains(&&chunk.chunk_type().bytes())
}

fn get_formatted_chunk_message(chunk: &Chunk) -> ColoredString {
    match chunk.data_as_string() {
        Ok(s) if s.trim().is_empty() => "* Blank *".yellow(),
        Ok(s) => s.normal(),
        Err(_) => "* Not valid UTF-8 *".yellow(),
    }
}
