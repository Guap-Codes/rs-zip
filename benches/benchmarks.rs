use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use rs_zip::{compression, decompression, file_io};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use rand::Rng;
// use std::path::PathBuf;

// Smaller sample size for faster benchmarks
const SAMPLE_SIZE_MB: usize = 10;  // Reduced from 50MB
const QUICK_MODE: bool = true;     // Set to false for full benchmarks

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(size);
    // Instead of unsafe set_len, fill the entire buffer safely.
    data.resize(size, 0);
    rng.fill(&mut data[..]);
    data
}

fn setup_xz_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("XZ Format");
    if QUICK_MODE {
        group.sample_size(10).sampling_mode(SamplingMode::Flat);
    }

    let sizes = if QUICK_MODE { 
        vec![1]  // Only test 1MB in quick mode
    } else { 
        vec![1, 10, 100]
    };

    for size in sizes {
        let data = generate_test_data(size * 1024 * 1024);
        // Compression benchmark with in-memory buffer
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(
            format!("Compress {size}MB"), 
            &data,
            |b, data| {
                b.iter_batched(
                    || std::io::Cursor::new(data.clone()),
                    |mut input| {
                        let mut output = Vec::with_capacity(data.len() / 2);
                        compression::compress_lzma(&mut input, &mut output, 6).unwrap();
                        output
                    },
                    criterion::BatchSize::PerIteration
                )
            }
        );

        // Prepare pre-compressed data for decompression benchmark
        let mut compressed = Vec::new();
        compression::compress_lzma(&mut std::io::Cursor::new(data.clone()), &mut compressed, 6).unwrap();
        
        group.bench_with_input(
            format!("Decompress {size}MB"), 
            &compressed,
            |b, compressed| {
                b.iter_batched(
                    || std::io::Cursor::new(compressed.clone()),
                    |mut input| {
                        let mut output = Vec::with_capacity(size * 1024 * 1024);
                        decompression::decompress_lzma(&mut input, &mut output).unwrap();
                        output
                    },
                    criterion::BatchSize::PerIteration
                )
            }
        );
    }
    group.finish();
}

fn setup_rsz_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("RSZ Format");
    if QUICK_MODE {
        group.sample_size(10).sampling_mode(SamplingMode::Flat);
    }

    // Instead of using in-memory cursors, write the generated file data to disk.
    // We'll create a temporary directory and create one file per entry.
    let counts = if QUICK_MODE {
        vec![10]  // Only test 10 files in quick mode
    } else {
        vec![10, 100, 500]
    };

    for count in counts {
        // Create a temporary directory to hold the test files.
        let temp_dir = TempDir::new().unwrap();
        let file_size = SAMPLE_SIZE_MB * 1024 * 1024 / count;

        // Write out "count" files to the temporary directory.
        let mut file_paths = Vec::new();
        for i in 0..count {
            let file_data = generate_test_data(file_size);
            let file_path = temp_dir.path().join(format!("file_{}.bin", i));
            File::create(&file_path)
                .unwrap()
                .write_all(&file_data)
                .unwrap();
            file_paths.push(file_path);
        }

        group.throughput(Throughput::Bytes((file_size * count) as u64));
        
        // Compression benchmark
        group.bench_function(
            &format!("Compress {count} files"), 
            |b| {
                b.iter(|| {
                    // Create a new archive in the temp directory.
                    let archive_path = temp_dir.path().join(format!("archive_{count}.rsz"));
                    // Pass the slice of file paths (PathBuf) to create_padded_archive.
                    compression::create_padded_archive(
                        &file_paths,
                        archive_path.to_str().unwrap(),
                        6
                    ).unwrap();
                })
            }
        );
        
        // For decompression, create the archive once.
        let archive_path = temp_dir.path().join(format!("archive_{count}.rsz"));
        compression::create_padded_archive(
            &file_paths,
            archive_path.to_str().unwrap(),
            6
        ).unwrap();
        
        // Decompression benchmark
        group.bench_function(
            &format!("Decompress {count} files"), 
            |b| {
                b.iter(|| {
                    decompression::extract_archive(&archive_path).unwrap();
                })
            }
        );
    }
    group.finish();
}

criterion_group!(benches, setup_xz_bench, setup_rsz_bench);
criterion_main!(benches);






/// This is a more robust benchmark test still in the works.
/// It runs the compression & decompression of both formats against 
/// larger number sample sets.
/*use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rs_zip::{compression, decompression, file_io};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use rand::Rng;

const SAMPLE_SIZE_MB: usize = 50;  // Total data size for benchmarks

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.r#gen::<u8>()).collect()
}

fn setup_xz_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("XZ Format");
    let temp_dir = TempDir::new().unwrap();
    
    let sizes = [1, 10, 100];
    for size in sizes {
        let data = generate_test_data(size * 1024 * 1024);
        let input_path = temp_dir.path().join(format!("{size}mb.bin"));
        File::create(&input_path).unwrap().write_all(&data).unwrap();

        group.throughput(Throughput::Bytes(data.len() as u64));
        
        // Compression benchmark
        group.bench_with_input(
            format!("Compress {size}MB"), 
            &input_path,
            |b, path| {
                b.iter(|| {
                    let mut input = File::open(path).unwrap();
                    let output = File::create(temp_dir.path().join("out.xz")).unwrap();
                    compression::compress_lzma(&mut input, output, 6).unwrap();
                })
            }
        );
        
        // Create fresh compressed file for decompression test
        let compressed_path = temp_dir.path().join(format!("compressed_{size}.xz"));
        compression::compress_lzma(
            &mut File::open(&input_path).unwrap(),
            File::create(&compressed_path).unwrap(),
            6
        ).unwrap();

        // Decompression benchmark
        group.bench_with_input(
            format!("Decompress {size}MB"), 
            &compressed_path,
            |b, path| {
                b.iter(|| {
                    let mut input = File::open(path).unwrap();
                    let mut output = File::create(temp_dir.path().join("decompressed.bin")).unwrap();
                    decompression::decompress_lzma(&mut input, &mut output).unwrap();
                })
            }
        );
    }
    group.finish();
}

fn setup_rsz_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("RSZ Format");
    let temp_dir = TempDir::new().unwrap();
    
    // Create test directory structure
    let dir_sizes = [10, 100, 500];  // Number of files
    for count in dir_sizes {
        let dir = temp_dir.path().join(format!("{count}_files"));
        std::fs::create_dir_all(&dir).unwrap();
        
        for i in 0..count {
            let mut file = File::create(dir.join(format!("file_{i}.bin"))).unwrap();
            file.write_all(&generate_test_data(SAMPLE_SIZE_MB * 1024 * 1024 / count)).unwrap();
        }

        let files = file_io::collect_files(&[PathBuf::from(&dir)], true).unwrap();
        let archive_path = temp_dir.path().join(format!("archive_{count}.rsz"));
        
        group.throughput(Throughput::Bytes(
            files.iter()
                .map(|p| p.metadata().unwrap().len())
                .sum()
        ));
        
        group.bench_with_input(
            format!("Compress {count} files"), 
            &files,
            |b, files| {
                b.iter(|| {
                    compression::create_padded_archive(
                        files,
                        &archive_path.to_string_lossy(),
                        6
                    ).unwrap();
                })
            }
        );
        
        group.bench_with_input(
            format!("Decompress {count} files"), 
            &archive_path,
            |b, path| {
                b.iter(|| {
                    decompression::extract_archive(path).unwrap();
                })
            }
        );
    }
    group.finish();
}

criterion_group!(benches, setup_xz_bench, setup_rsz_bench);
criterion_main!(benches);*/