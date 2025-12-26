//! Benchmarks for StarBreaker parsers
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::io::Cursor;

use starbreaker_parsers::p4k::{P4kCompression, CompressionMethod};

/// Benchmark compression methods
fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Create test data of various sizes
    let sizes = [1024, 10240, 102400, 1024000];

    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark Store (no compression)
        group.bench_with_input(
            BenchmarkId::new("store", size),
            &data,
            |b, data| {
                b.iter(|| {
                    P4kCompression::decompress(
                        black_box(data),
                        CompressionMethod::Store,
                        data.len(),
                    )
                })
            },
        );

        // Benchmark Deflate decompression
        // First compress the data
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("deflate", size),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    P4kCompression::decompress(
                        black_box(compressed),
                        CompressionMethod::Deflate,
                        size,
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark pattern matching
fn bench_pattern_matching(c: &mut Criterion) {
    use starbreaker_parsers::p4k::{P4kArchive, P4kEntry};
    use std::collections::HashMap;

    // Create a mock archive with many entries
    let mut entries = Vec::new();
    let patterns = ["Data/Objects/", "Data/Textures/", "Data/Sounds/", "Data/Libs/"];
    let extensions = ["cgf", "dds", "wem", "xml", "cfg"];

    for i in 0..10000 {
        let pattern = patterns[i % patterns.len()];
        let ext = extensions[i % extensions.len()];
        let path = format!("{}file_{}.{}", pattern, i, ext);
        
        entries.push(P4kEntry {
            path,
            compression: CompressionMethod::Store,
            crc32: 0,
            compressed_size: 1000,
            uncompressed_size: 2000,
            local_header_offset: 0,
            flags: 0,
            mod_time: 0,
            mod_date: 0,
            is_encrypted: false,
            is_directory: false,
        });
    }

    let mut path_index = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        path_index.insert(entry.path.clone(), idx);
    }
    
    let archive = P4kArchive { entries, path_index };

    let mut group = c.benchmark_group("pattern_matching");

    group.bench_function("find_by_extension", |b| {
        b.iter(|| archive.find_by_extension(black_box("dds")))
    });

    group.bench_function("find_wildcard", |b| {
        b.iter(|| archive.find(black_box("Data/Objects/*.cgf")))
    });

    group.bench_function("find_partial", |b| {
        b.iter(|| archive.find(black_box("file_500")))
    });

    group.bench_function("get_by_path", |b| {
        b.iter(|| archive.get(black_box("Data/Objects/file_500.cgf")))
    });

    group.finish();
}

/// Benchmark tree building
fn bench_tree_building(c: &mut Criterion) {
    use starbreaker_parsers::p4k::{P4kArchive, P4kEntry};
    use std::collections::HashMap;

    // Create archive with realistic path hierarchy
    let mut entries = Vec::new();
    
    // Create directory structure
    let dirs = [
        "Data/",
        "Data/Objects/",
        "Data/Objects/Spaceships/",
        "Data/Objects/Spaceships/MISC/",
        "Data/Objects/Spaceships/ORIG/",
        "Data/Textures/",
        "Data/Libs/",
    ];

    for dir in &dirs {
        entries.push(P4kEntry {
            path: dir.to_string(),
            compression: CompressionMethod::Store,
            crc32: 0,
            compressed_size: 0,
            uncompressed_size: 0,
            local_header_offset: 0,
            flags: 0,
            mod_time: 0,
            mod_date: 0,
            is_encrypted: false,
            is_directory: true,
        });
    }

    // Add files
    for i in 0..5000 {
        let base = dirs[i % dirs.len()];
        if !base.ends_with('/') { continue; }
        
        entries.push(P4kEntry {
            path: format!("{}file_{}.cgf", base, i),
            compression: CompressionMethod::Store,
            crc32: 0,
            compressed_size: 1000,
            uncompressed_size: 2000,
            local_header_offset: 0,
            flags: 0,
            mod_time: 0,
            mod_date: 0,
            is_encrypted: false,
            is_directory: false,
        });
    }

    let mut path_index = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        path_index.insert(entry.path.clone(), idx);
    }
    
    let archive = P4kArchive { entries, path_index };

    c.bench_function("build_tree", |b| {
        b.iter(|| archive.build_tree())
    });
}

criterion_group!(
    benches,
    bench_compression,
    bench_pattern_matching,
    bench_tree_building,
);

criterion_main!(benches);