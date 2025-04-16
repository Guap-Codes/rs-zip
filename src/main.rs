pub mod cli;
pub mod compression;
pub mod decompression;
pub mod file_io;

use cli::Cli;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    let args = Cli::parse_args();
    args.validate()?;

    if args.decompress {
        if args.format.to_lowercase() == "rsz" {
            // RSZ mode: use the archive input from args.inputs[0]
            let archive_path = args.inputs.first().unwrap().to_str().unwrap();
            decompression::extract_archive(archive_path)
        } else {
            // XZ mode: if no output is provided, derive a default one.
            let input_path = args.inputs.first().unwrap().to_str().unwrap();
            let output_path = match &args.output {
                Some(path) => path.to_str().unwrap().to_string(),
                None => crate::file_io::default_output_path(input_path, false),
            };
            let input_file = File::open(input_path)?;
            let mut output_file = file_io::create_output_file(&output_path)?;
            decompression::decompress_lzma(input_file, &mut output_file)
        }
    } else {
        // Compression branch
        if args.format.to_lowercase() == "rsz" {
            // RSZ format: support multiple files using the custom archive format.
            let files = file_io::collect_files(&args.inputs, args.recursive)?;
            let output_path = args.output.as_ref().unwrap().to_str().unwrap();
            compression::create_padded_archive(&files, output_path, args.level)
        } else {
            // XZ format: compress a single file.
            let input_path = args.inputs.first().unwrap().to_str().unwrap();
            let mut input_file = File::open(input_path)?;
            let output_path = args.output.as_ref().unwrap().to_str().unwrap();
            let output_file = file_io::create_output_file(output_path)?;
            compression::compress_lzma(&mut input_file, output_file, args.level)
        }
    }
}





/*// src/main.rs
mod cli;
mod compression;
mod decompression;
mod file_io;

use cli::Cli;
use std::fs::File;
use std::io::{self, Read, Write, Cursor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

fn main() -> io::Result<()> {
    // Parse and validate command line arguments
    let args = Cli::parse_args();
    args.validate()?;

    // Convert paths to strings
    let input_path = args.input.to_str().ok_or_else(|| 
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid input path encoding"))?;
    let output_path = args.output.as_ref().unwrap().to_str().ok_or_else(|| 
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid output path encoding"))?;

    // Open input file
    let mut input_file = File::open(input_path)?;
    let mut output_file = file_io::create_output_file(output_path)?;

    if args.decompress {
        // Read original size prefix (8 bytes)
        let mut size_buf = [0u8; 8];
        input_file.read_exact(&mut size_buf)?;
        let original_size = u64::from_le_bytes(size_buf);

        // Decompress to buffer
        let mut decompressed_data = Vec::new();
        decompression::decompress_lzma(&mut input_file, &mut decompressed_data)?;

        // Truncate to original size
        decompressed_data.truncate(original_size as usize);

        // Write to output
        output_file.write_all(&decompressed_data)?;
        output_file.flush()?;
        Ok(())
    } else {
        // Read input into buffer
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        let original_size = buffer.len() as u64;

        // Pad if necessary
        let pad_threshold = 64;
        if buffer.len() < pad_threshold {
            buffer.resize(pad_threshold, 0);
        }

        // Write original size prefix
        output_file.write_u64::<LittleEndian>(original_size)?;

        // Compress with size-padded input
        let mut padded_input = Cursor::new(buffer);
        compression::compress_lzma(&mut padded_input, output_file, args.level)
    }
}*/