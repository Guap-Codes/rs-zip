use std::fs::File;
use std::io::{self, Read, Write};
use xz2::write::XzEncoder;
use byteorder::{WriteBytesExt, LittleEndian};
use std::path::PathBuf;
use std::io::Cursor;


const PAD_THRESHOLD: usize = 64;


/// Compresses data from the input reader and writes it to the output writer using LZMA.
/// 
/// For small files, this implementation pads the input to at least PAD_THRESHOLD bytes,
/// writes an 8-byte header with the original input size (in Little Endian), and then
/// compresses the padded data.
///
/// # Arguments
/// * `input` - A reader implementing the Read trait for uncompressed data.
/// * `output` - A writer implementing the Write trait for compressed data.
/// * `compression_level` - Compression strength (0-9, where 9 is maximum compression).
///
/// # Returns
/// * `Result<(), std::io::Error>` - Ok on success, Io error on failure.
pub fn compress_lzma<R: Read, W: Write>(
    input: &mut R,
    mut output: W,
    compression_level: u32,
) -> Result<(), std::io::Error> {
    if compression_level > 9 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Compression level must be between 0 and 9",
        ));
    }

    // Read the entire input into memory.
    let mut data = Vec::new();
    input.read_to_end(&mut data)?;
    let original_size = data.len() as u64;

    // Pad if the data is smaller than the threshold.
    if data.len() < PAD_THRESHOLD {
        data.resize(PAD_THRESHOLD, 0);
    }

    // Write an 8-byte header with the original (unpadded) size.
    output.write_u64::<LittleEndian>(original_size)?;

    // Create the XZ encoder that will compress the padded data.
    let mut encoder = XzEncoder::new(output, compression_level);

    // Use a cursor to read from the padded data buffer.
    let mut cursor = Cursor::new(data);
    std::io::copy(&mut cursor, &mut encoder)?;

    // Finalize the encoder and flush.
    let mut final_output = encoder.finish()?;
    final_output.flush()?;
    Ok(())
}

/// Creates a custom RSZ archive from a list of files with padding and original size metadata.
///
/// # Arguments
/// * `files` - Slice of PathBufs representing the input files.
/// * `output_path` - Path to the output archive file.
/// * `compression_level` - Compression strength (0–9).
///
/// # Returns
/// * `Result<(), io::Error>` - Ok on success or an error.
pub fn create_padded_archive(
    files: &[PathBuf],
    output_path: &str,
    compression_level: u32,
) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    let mut encoder = XzEncoder::new(output_file, compression_level);

    let valid_files: Vec<&PathBuf> = files
        .iter()
        .filter(|p| p.to_str().is_some())
        .collect();

    encoder.write_u64::<LittleEndian>(valid_files.len() as u64)?;

    for file_path in valid_files {
        let file_name = file_path.to_str().unwrap();
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let original_size = buffer.len() as u64;

        // Write metadata
        encoder.write_u64::<LittleEndian>(file_name.len() as u64)?;
        encoder.write_all(file_name.as_bytes())?;
        encoder.write_u64::<LittleEndian>(original_size)?;

        // Write original content WITHOUT padding
        let mut cursor = io::Cursor::new(buffer);
        io::copy(&mut cursor, &mut encoder)?;
    }

    let mut output = encoder.finish()?;
    output.flush()?;
    Ok(())
}