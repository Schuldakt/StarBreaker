//! Unit tests for the P4K parser
//!
//! These tests cover the P4K archive parsing functionality including:
//! - Entry parsing and metadata extraction
//! - Pattern matching and glob search
//! - Directory tree building
//! - Compression method detection
//! - Archive statistics

use std::collections::HashMap;
use std::io::Cursor;

use crate::p4k::{
    P4kParser, P4kArchive, P4kEntry, P4kEntryInfo,
    CompressionMethod, DirectoryNode, P4kCompression,
};
use crate::traits::{Parser, RandomAccessParser, ParseOptions};

/// Helper to create a test entry
fn make_entry(path: &str, compressed: u64, uncompressed: u64, is_dir: bool) -> P4kEntry {
    P4kEntry {
        path: path.to_string(),
        compression: CompressionMethod::Store,
        crc32: 0xDEADBEEF,
        compressed_size: compressed,
        uncompressed_size: uncompressed,
        local_header_offset: 0,
        flags: 0,
        mod_time: 0x4800, // 9:00 AM
        mod_date: 0x4E21, // Jan 1, 2019
        is_encrypted: false,
        is_directory: is_dir,
    }
}

/// Helper to create a test archive with sample entries
fn make_test_archive() -> P4kArchive {
    let entries = vec![
        make_entry("Data/", 0, 0, true),
        make_entry("Data/Libs/", 0, 0, true),
        make_entry("Data/Libs/Config/", 0, 0, true),
        make_entry("Data/Libs/Config/defaultprofile.xml", 1024, 4096, false),
        make_entry("Data/Libs/Config/game.cfg", 512, 2048, false),
        make_entry("Data/Textures/", 0, 0, true),
        make_entry("Data/Textures/ship_diff.dds", 1048576, 4194304, false),
        make_entry("Data/Textures/ship_norm.dds", 524288, 2097152, false),
        make_entry("Data/Objects/", 0, 0, true),
        make_entry("Data/Objects/Spaceships/", 0, 0, true),
        make_entry("Data/Objects/Spaceships/aurora.cgf", 262144, 1048576, false),
        make_entry("Data/Objects/Spaceships/constellation.cgf", 524288, 2097152, false),
        make_entry("Data/Sounds/", 0, 0, true),
        make_entry("Data/Sounds/engine.wem", 131072, 524288, false),
    ];

    let mut path_index = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        path_index.insert(entry.path.clone(), idx);
    }

    P4kArchive { entries, path_index }
}

mod entry_tests {
    use super::*;

    #[test]
    fn test_filename_extraction() {
        let entry = make_entry("Data/Libs/Config/defaultprofile.xml", 100, 200, false);
        assert_eq!(entry.filename(), "defaultprofile.xml");
    }

    #[test]
    fn test_filename_root_file() {
        let entry = make_entry("readme.txt", 100, 200, false);
        assert_eq!(entry.filename(), "readme.txt");
    }

    #[test]
    fn test_filename_directory() {
        let entry = make_entry("Data/Libs/", 0, 0, true);
        // Directory names end with /, so filename strips trailing /
        assert_eq!(entry.filename(), "Libs");
    }

    #[test]
    fn test_parent_path() {
        let entry = make_entry("Data/Libs/Config/defaultprofile.xml", 100, 200, false);
        assert_eq!(entry.parent(), Some("Data/Libs/Config"));
    }

    #[test]
    fn test_parent_root() {
        let entry = make_entry("readme.txt", 100, 200, false);
        assert_eq!(entry.parent(), None);
    }

    #[test]
    fn test_extension() {
        let entry = make_entry("model.cgf", 100, 200, false);
        assert_eq!(entry.extension(), Some("cgf"));
    }

    #[test]
    fn test_extension_multiple_dots() {
        let entry = make_entry("texture.dds.1", 100, 200, false);
        assert_eq!(entry.extension(), Some("1"));
    }

    #[test]
    fn test_extension_directory() {
        let entry = make_entry("Data/", 0, 0, true);
        assert_eq!(entry.extension(), None);
    }

    #[test]
    fn test_compression_ratio() {
        let entry = make_entry("file.txt", 100, 400, false);
        assert!((entry.compression_ratio() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_compression_ratio_zero_size() {
        let entry = make_entry("empty.txt", 0, 0, false);
        assert!((entry.compression_ratio() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_modification_datetime() {
        let entry = make_entry("file.txt", 100, 200, false);
        let (year, month, day, hour, minute, second) = entry.modification_datetime();
        
        // Verify the DOS datetime decoding
        assert!(year >= 1980);
        assert!((1..=12).contains(&month));
        assert!((1..=31).contains(&day));
        assert!(hour < 24);
        assert!(minute < 60);
        assert!(second < 60);
    }
}

mod archive_tests {
    use super::*;

    #[test]
    fn test_entry_count() {
        let archive = make_test_archive();
        assert_eq!(archive.entry_count(), 14);
    }

    #[test]
    fn test_file_count() {
        let archive = make_test_archive();
        assert_eq!(archive.file_count(), 7);
    }

    #[test]
    fn test_directory_count() {
        let archive = make_test_archive();
        assert_eq!(archive.directory_count(), 7);
    }

    #[test]
    fn test_total_sizes() {
        let archive = make_test_archive();
        assert!(archive.total_uncompressed_size() > 0);
        assert!(archive.total_compressed_size() > 0);
        assert!(archive.total_compressed_size() <= archive.total_uncompressed_size());
    }

    #[test]
    fn test_get_existing_entry() {
        let archive = make_test_archive();
        let entry = archive.get("Data/Libs/Config/defaultprofile.xml");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().uncompressed_size, 4096);
    }

    #[test]
    fn test_get_nonexistent_entry() {
        let archive = make_test_archive();
        assert!(archive.get("nonexistent.txt").is_none());
    }

    #[test]
    fn test_contains() {
        let archive = make_test_archive();
        assert!(archive.contains("Data/Textures/ship_diff.dds"));
        assert!(!archive.contains("Data/Textures/missing.dds"));
    }
}

mod pattern_matching_tests {
    use super::*;

    #[test]
    fn test_find_by_extension_dds() {
        let archive = make_test_archive();
        let results = archive.find_by_extension("dds");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.path.ends_with(".dds")));
    }

    #[test]
    fn test_find_by_extension_cgf() {
        let archive = make_test_archive();
        let results = archive.find_by_extension(".cgf");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_by_extension_case_insensitive() {
        let archive = make_test_archive();
        let results_lower = archive.find_by_extension("dds");
        let results_upper = archive.find_by_extension("DDS");
        assert_eq!(results_lower.len(), results_upper.len());
    }

    #[test]
    fn test_find_wildcard_all_xml() {
        let archive = make_test_archive();
        let results = archive.find("*.xml");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_wildcard_path() {
        let archive = make_test_archive();
        let results = archive.find("Data/Objects/Spaceships/*");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_wildcard_extension() {
        let archive = make_test_archive();
        let results = archive.find("*.cgf");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_partial_match() {
        let archive = make_test_archive();
        let results = archive.find("ship");
        // Should match ship_diff.dds, ship_norm.dds
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_find_no_matches() {
        let archive = make_test_archive();
        let results = archive.find("*.nonexistent");
        assert!(results.is_empty());
    }
}

mod directory_listing_tests {
    use super::*;

    #[test]
    fn test_list_root_directory() {
        let archive = make_test_archive();
        let results = archive.list_directory("");
        assert_eq!(results.len(), 1); // Only "Data/"
        assert_eq!(results[0].path, "Data/");
    }

    #[test]
    fn test_list_data_directory() {
        let archive = make_test_archive();
        let results = archive.list_directory("Data");
        // Should contain Libs/, Textures/, Objects/, Sounds/
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_list_config_directory() {
        let archive = make_test_archive();
        let results = archive.list_directory("Data/Libs/Config");
        assert_eq!(results.len(), 2); // defaultprofile.xml, game.cfg
    }

    #[test]
    fn test_list_nonexistent_directory() {
        let archive = make_test_archive();
        let results = archive.list_directory("Nonexistent");
        assert!(results.is_empty());
    }
}

mod tree_building_tests {
    use super::*;

    #[test]
    fn test_build_tree() {
        let archive = make_test_archive();
        let tree = archive.build_tree();
        
        assert!(tree.children.contains_key("Data"));
    }

    #[test]
    fn test_tree_structure() {
        let archive = make_test_archive();
        let tree = archive.build_tree();
        
        let data = &tree.children["Data"];
        assert!(data.children.contains_key("Libs"));
        assert!(data.children.contains_key("Textures"));
        assert!(data.children.contains_key("Objects"));
        assert!(data.children.contains_key("Sounds"));
    }

    #[test]
    fn test_tree_file_nodes() {
        let archive = make_test_archive();
        let tree = archive.build_tree();
        
        let config = &tree.children["Data"].children["Libs"].children["Config"];
        assert!(config.children.contains_key("defaultprofile.xml"));
        assert!(config.children["defaultprofile.xml"].is_file);
    }

    #[test]
    fn test_directory_node_insert() {
        let mut root = DirectoryNode::new("root".to_string());
        root.insert("a/b/c/file.txt", false);
        
        assert!(root.children.contains_key("a"));
        assert!(root.children["a"].children.contains_key("b"));
        assert!(root.children["a"].children["b"].children.contains_key("c"));
        assert!(root.children["a"].children["b"].children["c"].children.contains_key("file.txt"));
        assert!(root.children["a"].children["b"].children["c"].children["file.txt"].is_file);
    }

    #[test]
    fn test_sorted_children() {
        let mut root = DirectoryNode::new("root".to_string());
        root.insert("zebra.txt", false);
        root.insert("alpha.txt", false);
        root.insert("mango.txt", false);
        
        let sorted = root.sorted_children();
        assert_eq!(sorted[0], "alpha.txt");
        assert_eq!(sorted[1], "mango.txt");
        assert_eq!(sorted[2], "zebra.txt");
    }
}

mod compression_tests {
    use super::*;

    #[test]
    fn test_compression_method_from_u16() {
        assert_eq!(CompressionMethod::from(0), CompressionMethod::Store);
        assert_eq!(CompressionMethod::from(8), CompressionMethod::Deflate);
        assert_eq!(CompressionMethod::from(93), CompressionMethod::Zstd);
        assert_eq!(CompressionMethod::from(99), CompressionMethod::Lz4);
    }

    #[test]
    fn test_compression_method_unknown() {
        let method = CompressionMethod::from(255);
        assert!(matches!(method, CompressionMethod::Unknown(255)));
    }

    #[test]
    fn test_store_decompression() {
        let data = vec![1, 2, 3, 4, 5];
        let result = P4kCompression::decompress(&data, CompressionMethod::Store, 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_deflate_decompression() {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let original = b"Hello, StarBreaker!";
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();
        
        let result = P4kCompression::decompress(&compressed, CompressionMethod::Deflate, original.len());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original);
    }
}

mod statistics_tests {
    use super::*;

    #[test]
    fn test_statistics_counts() {
        let archive = make_test_archive();
        let stats = archive.statistics();
        
        assert_eq!(stats.total_entries, 14);
        assert_eq!(stats.file_count, 7);
        assert_eq!(stats.directory_count, 7);
    }

    #[test]
    fn test_statistics_extensions() {
        let archive = make_test_archive();
        let stats = archive.statistics();
        
        assert_eq!(stats.extensions.get("dds"), Some(&2));
        assert_eq!(stats.extensions.get("cgf"), Some(&2));
        assert_eq!(stats.extensions.get("xml"), Some(&1));
        assert_eq!(stats.extensions.get("cfg"), Some(&1));
        assert_eq!(stats.extensions.get("wem"), Some(&1));
    }

    #[test]
    fn test_statistics_compression_ratio() {
        let archive = make_test_archive();
        let stats = archive.statistics();
        
        // Ratio should be between 0 and 1 since compressed < uncompressed
        assert!(stats.compression_ratio > 0.0);
        assert!(stats.compression_ratio < 1.0);
    }
}

mod entry_info_tests {
    use super::*;

    #[test]
    fn test_formatted_size_bytes() {
        let info = P4kEntryInfo {
            path: "small.txt".to_string(),
            compressed_size: 100,
            uncompressed_size: 512,
            is_directory: false,
            compression: CompressionMethod::Store,
        };
        
        assert_eq!(info.formatted_size(), "512 B");
    }

    #[test]
    fn test_formatted_size_kilobytes() {
        let info = P4kEntryInfo {
            path: "medium.txt".to_string(),
            compressed_size: 1024,
            uncompressed_size: 2048,
            is_directory: false,
            compression: CompressionMethod::Store,
        };
        
        assert_eq!(info.formatted_size(), "2.00 KB");
    }

    #[test]
    fn test_formatted_size_megabytes() {
        let info = P4kEntryInfo {
            path: "large.dds".to_string(),
            compressed_size: 1048576,
            uncompressed_size: 4194304,
            is_directory: false,
            compression: CompressionMethod::Store,
        };
        
        assert_eq!(info.formatted_size(), "4.00 MB");
    }

    #[test]
    fn test_formatted_size_gigabytes() {
        let info = P4kEntryInfo {
            path: "huge.pak".to_string(),
            compressed_size: 1073741824,
            uncompressed_size: 2147483648,
            is_directory: false,
            compression: CompressionMethod::Store,
        };
        
        assert_eq!(info.formatted_size(), "2.00 GB");
    }
}

mod parser_trait_tests {
    use super::*;

    #[test]
    fn test_parser_extensions() {
        let parser = P4kParser::new();
        let extensions = parser.extensions();
        
        assert!(extensions.contains(&"p4k"));
    }

    #[test]
    fn test_parser_name() {
        let parser = P4kParser::new();
        assert!(!parser.name().is_empty());
    }

    #[test]
    fn test_parser_magic_bytes() {
        let parser = P4kParser::new();
        let magic = parser.magic_bytes();
        
        assert!(magic.is_some());
        assert_eq!(magic.unwrap(), &[0x50, 0x4B, 0x03, 0x04]); // "PK\x03\x04"
    }
}

// Property-based tests using proptest
#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_entry_filename_never_empty(path in "[a-zA-Z0-9_/]+\\.[a-z]{3}") {
            let entry = make_entry(&path, 100, 200, false);
            prop_assert!(!entry.filename().is_empty());
        }

        #[test]
        fn test_entry_extension_for_files(ext in "[a-z]{2,4}") {
            let path = format!("test/file.{}", ext);
            let entry = make_entry(&path, 100, 200, false);
            prop_assert_eq!(entry.extension(), Some(ext.as_str()));
        }

        #[test]
        fn test_compression_ratio_bounds(compressed in 1u64..1000000, uncompressed in 1u64..1000000) {
            let entry = make_entry("file.txt", compressed, uncompressed, false);
            let ratio = entry.compression_ratio();
            prop_assert!(ratio > 0.0);
            prop_assert!(ratio.is_finite());
        }

        #[test]
        fn test_find_pattern_doesnt_panic(pattern in "[a-zA-Z0-9*_.]+") {
            let archive = make_test_archive();
            // Should not panic regardless of pattern
            let _ = archive.find(&pattern);
        }
    }
}

// Integration-style tests (would need actual P4K files)
#[cfg(test)]
mod integration_tests {
    use super::*;

    // This test requires an actual P4K file
    #[test]
    #[ignore = "requires Star Citizen installation"]
    fn test_parse_real_archive() {
        let parser = P4kParser::new();
        
        // Try common installation paths
        let paths = [
            "C:\\Program Files\\Roberts Space Industries\\StarCitizen\\LIVE\\Data.p4k",
            "D:\\Games\\StarCitizen\\LIVE\\Data.p4k",
        ];

        for path in &paths {
            let path = std::path::Path::new(path);
            if path.exists() {
                let result = parser.parse_file(path);
                assert!(result.is_ok(), "Failed to parse {}: {:?}", path.display(), result.err());
                
                let archive = result.unwrap();
                assert!(archive.entry_count() > 0);
                println!("Parsed {} entries from {}", archive.entry_count(), path.display());
                return;
            }
        }

        panic!("No Star Citizen installation found for integration test");
    }
}