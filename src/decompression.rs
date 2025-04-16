use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use xz2::read::XzDecoder;
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::BufReader;
use std::fs;

/// Decompresses data from the input reader and writes the original (unpadded) content
/// to the output writer. The function expects an 8-byte header at the beginning of the stream
/// that represents the original unpadded size in Little Endian format.
/// 
/// # Arguments
/// * `input` - A reader implementing the Read trait for compressed data.
/// * `output` - A mutable reference to a writer implementing the Write trait for decompressed data.
/// 
/// # Returns
/// * `Result<(), std::io::Error>` - Ok on success, or an Io error on failure.
pub fn decompress_lzma<R: Read, W: Write>(
    mut input: R,
    output: &mut W,
) -> Result<(), std::io::Error> {
    // Read the first 8 bytes of the input as the original unpadded size.
    let original_size = input.read_u64::<LittleEndian>()?;
    
    // Create an XZ decoder for the remaining input stream.
    let mut decoder = XzDecoder::new(input);
    
    // Decompress the data into a buffer.
    let mut decompressed_data = Vec::new();
    std::io::copy(&mut decoder, &mut decompressed_data)?;
    
    // Truncate the decompressed data to the original size.
    output.write_all(&decompressed_data[..(original_size as usize)])?;
    output.flush()?;
    
    Ok(())
}


/// Extracts an archive compressed using RSZ format (LZMA with custom metadata).
/// This function reads the number of files from the archive header, then iterates
/// through each file, reading its name, original size, and content (with padding),
/// and writes the original data to disk.
///
/// # Arguments
/// * `archive_path` - Path to the `.rsz` archive file.
///
/// # Returns
/// * `Result<(), std::io::Error>` - Ok if all files are extracted successfully.
pub fn extract_archive<P: AsRef<Path>>(archive_path: P) -> io::Result<()> {
    let file = File::open(archive_path)?;
    let decoder = XzDecoder::new(file);
    let mut reader = BufReader::new(decoder);

    let file_count = reader.read_u64::<LittleEndian>()?;

    for _ in 0..file_count {
        let name_len = reader.read_u64::<LittleEndian>()? as usize;
        let mut name_buf = vec![0u8; name_len];
        reader.read_exact(&mut name_buf)?;
        let file_name = String::from_utf8_lossy(&name_buf).into_owned();
        
        let original_size = reader.read_u64::<LittleEndian>()?;
        let mut content = vec![0u8; original_size as usize];
        reader.read_exact(&mut content)?;

        // Create parent directories
        if let Some(parent) = Path::new(&file_name).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut output_file = File::create(&file_name)?;
        output_file.write_all(&content)?;
    }

    Ok(())
}