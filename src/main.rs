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