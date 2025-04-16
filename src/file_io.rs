use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Result};
use std::path::Path;
use walkdir::WalkDir;
use std::path::PathBuf;

/// Opens a file for reading with buffered I/O
/// 
/// # Arguments
/// * `path` - Path to the input file
/// 
/// # Returns
/// * `Result<BufReader<File>>` - Buffered reader for efficient file access
#[allow(dead_code)]
pub fn open_input_file(path: &str) -> Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

/// Creates a file for writing with buffered I/O
/// 
/// # Arguments
/// * `path` - Path to the output file
/// 
/// # Returns
/// * `Result<BufWriter<File>>` - Buffered writer for efficient file access
pub fn create_output_file(path: &str) -> Result<BufWriter<File>> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    Ok(BufWriter::new(file))
}


/// Validates that the input path exists and is a file
/// 
/// # Arguments
/// * `path` - Path to validate
/// 
/// # Returns
/// * `Result<()>` - Ok if valid, error otherwise
pub fn validate_input_path(path: &str) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input path must be a file",
        ));
    }
    Ok(())
}

/// Generates a default output path based on operation type
/// 
/// # Arguments
/// * `input_path` - Original file path
/// * `is_compression` - True for compression, false for decompression
/// 
/// # Returns
/// * `String` - Generated output path
pub fn default_output_path(input_path: &str, is_compression: bool) -> String {
    if is_compression {
        format!("{}.xz", input_path)
    } else {
        input_path
            .trim_end_matches(".xz")
            .trim_end_matches(".lzma")
            .to_string()
    }
}


/// Recursively collect file paths
pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for path in paths {
        if path.is_dir() {
            let walker = WalkDir::new(path).min_depth(1);
            let walker = if recursive { walker } else { walker.max_depth(1) };
            
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }
        } else if path.is_file() {
            files.push(path.clone());
        }
    }
    
    Ok(files)
}

/// Create parent directories for a path
#[allow(dead_code)]
pub fn ensure_parent_dir(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}