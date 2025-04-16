// src/cli.rs
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rs-zip")]
#[command(version = "1.0")]
#[command(about = "LZMA compression/decompression tool with custom RSZ archive option", long_about = None)]
pub struct Cli {
    /// Input file/directory paths
    #[arg(short, long, required = true, num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file path (optional, required for xz compression/decompression)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Decompress the input file/archive
    #[arg(short, long, default_value_t = false)]
    pub decompress: bool,

    /// Compression level (0-9)
    #[arg(short, long, default_value_t = 6, value_parser = clap::value_parser!(u32).range(0..=9))]
    pub level: u32,

    /// Recursively compress a directory (when inputs are directories)
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    /// Archive format: "xz" for single-file LZMA or "rsz" for the custom multi-file format
    #[arg(short, long, default_value = "xz")]
    pub format: String,
}

impl Cli {
    pub fn parse_args() -> Self {
        let mut args = Cli::parse();

        // When compressing (not decompressing) and no output is specified, generate a default output path.
        if args.output.is_none() && !args.decompress {
            let first_input = args.inputs.first().unwrap();
            let input_str = first_input.to_str().unwrap_or_default();
            let default_path = crate::file_io::default_output_path(input_str, true);
            args.output = Some(PathBuf::from(default_path));
        }
        args
    }

    pub fn validate(&self) -> std::io::Result<()> {
        for input in &self.inputs {
            let input_str = input.to_str().unwrap_or_default();
            crate::file_io::validate_input_path(input_str)?;
        }
        // For xz compression/decompression, we expect an output file. (For RSZ archives,
        // the output is built into the archive so that isn’t strictly needed on decompression.)
        if !self.decompress {
            if let Some(output_path) = &self.output {
                let _ = std::fs::File::create(output_path)?;
            }
        }
        Ok(())
    }
}
