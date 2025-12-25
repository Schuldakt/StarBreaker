// starbreaker-parsers/src/p4k/mod.rs
//! P4K Archive Parser
//! 
//! The P4K format is Star Citizen's main archive format, derived from the
//! ZIP format but with modifications. It contains compressed game assets
//! including textures, models, sounds, and data files.
//! 
//! # Format Structure
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! |                         P4K Archive                         |
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │              File Data (Compressed)                     ││
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐                    ││
//! │  │  │ File 1  │ │ File 2  │ │ File N  │ ...                ││
//! │  │  └─────────┘ └─────────┘ └─────────┘                    ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │           Central Directory (Entries)                   ││
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐                    ││
//! │  │  │ Entry 1 │ │ Entry 2 │ │ Entry N │ ...                ││
//! │  │  └─────────┘ └─────────┘ └─────────┘                    ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │            End of Central Directory                     ││
//! │  │  - Signature (0x06054B50)                               ││
//! │  │  - Central Directory offset                             ││
//! │  │  - Total entries count                                  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod archive;
mod entry;
mod compression;

pub use archive::P4kArchive;
pub use entry::{P4kEntry, P4kEntryInfo};
pub use compression::P4kCompression;

use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::{
    Parser, RandomAccessParser, ParseResult, ParseError,
    ParseOptions, ParseProgress, ParsePhase, ProgressCallback
};

/// Magic bytes for ZIP-based P4K format
const P4K_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04]; // "PK\x03\x04"

/// End of central directory signature
const EOCD_SIGNATURE: u32 = 0x06054B50;

/// Central directory file header signature
const CD_SIGNATURE: u32 = 0x02014B50;

/// Local file header signature
const LOCAL_HEADER_SIGNATURE: u32 = 0x04034B50;

/// ZIP64 end of central directory locator signature
const ZIP64_EOCD_SIGNATURE: u32 = 0x06064B50;

/// ZIP64 end of central directory locator signature
const ZIP64_EOCD_LOCATOR_SIGNATURE: u32 = 0x07064B50;

/// Compression methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CompressionMethod {
    Store = 0,
    Deflate = 8,
    Zstd = 93,
    Lz4 = 99, // Custom for Star Citizen
    Unknown(u16),
}

impl From<u16> for CompressionMethod {
    fn from(value: u16) -> Self {
        match value {
            0 => CompressionMethod::Store,
            8 => CompressionMethod::Deflate,
            93 => CompressionMethod::Zstd,
            99 => CompressionMethod::Lz4,
            other => CompressionMethod::Unknown(other),
        }
    }
}

/// P4K Archive Parser
/// 
/// Parses Star Citizen's P4K archive fomat, providing both full archive
/// parsing and random access to individual entries.
pub struct P4kParser {
    /// Cache of parsed archives by path
    cache: parking_lot::RwLock<HashMap<String, Arc<P4kArchive>>>,
}

impl P4kParser {
    /// Create a new P4K parser
    pub fn new() -> Self {
        Self {
            cache: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Parse the end of central directory recrod
    fn parse_eocd<R: Read + Seek>(&self, reader: &mut R) -> ParseResult<EndOfCentralDirectory> {
        // Seek to end and search backwards for EOCD signature
        let file_size = reader.seek(SeekFrom::End(0))?;

        // EOCD is at least 22 bytes, search within last 65KB for comment
        let search_start = file_size.saturating_sub(65536 + 22);
        reader.seek(SeekFrom::Start(search_start))?;

        let mut buffer = vec![0u8; (file_size - search_start) as usize];
        reader.read_exact(&mut buffer)?;

        // Search for EOCD signature from end
        let sig_bytes = EOCD_SIGNATURE.to_le_bytes();
        let eocd_offset = buffer.windows(4)
            .rposition(|w| w == sig_bytes)
            .ok_or_else(|| ParseError::InvalidMagic {
                expected: sig_bytes.to_vec(),
                found: vec![],
            })?;

        let eocd_abs_offset = search_start + eocd_offset as u64;
        reader.seek(SeekFrom::Start(eocd_abs_offset))?;

        // Parse EOCD
        let mut eocd_data = [0u8; 22];
        reader.read_exact(&mut eocd_data)?;

        let disk_number     = u16::from_le_bytes([eocd_data[4], eocd_data[5]]);
        let cd_disk         = u16::from_le_bytes([eocd_data[6], eocd_data[7]]);
        let disk_entries    = u16::from_le_bytes([eocd_data[8], eocd_data[9]]);
        let total_entries   = u16::from_le_bytes([eocd_data[10], eocd_data[11]]);
        let cd_size         = u32::from_le_bytes([eocd_data[12], eocd_data[13], eocd_data[14], eocd_data[15]]);
        let cd_offset       = u32::from_le_bytes([eocd_data[16], eocd_data[17], eocd_data[18], eocd_data[19]]);
        let comment_length  = u16::from_le_bytes([eocd_data[20], eocd_data[21]]);

        // Check for ZIP64
        let (cd_offset, total_entries) = if cd_offset == 0xFFFFFFFF || total_entries == 0xFFFF {
            self.parse_zip64_eocd(reader, eocd_abs_offset)?
        } else {
            (cd_offset as u64, total_entries as u64)
        };

        Ok(EndOfCentralDirectory {
            disk_number,
            cd_disk,
            disk_entries: disk_entries as u64,
            total_entries,
            cd_size: cd_size as u64,
            cd_offset,
            comment_length,
        })

    }

    /// Parse ZIP64 end of central directory
    fn parse_zip64_eocd<R: Read + Seek>(
        &self,
        reader: &mut R,
        eocd_offset: u64
    ) -> ParseResult<(u64, u64)> {
        // Look for ZIP64 EOCD locator before EOCD
        let locator_offset = eocd_offset.saturating_sub(20);
        reader.seek(SeekFrom::Start(locator_offset))?;

        let mut locator = [0u8; 20];
        reader.read_exact(&mut locator)?;

        let sig = u32::from_le_bytes([locator[0], locator[1], locator[2], locator[3]]);
        if sig != ZIP64_EOCD_LOCATOR_SIGNATURE {
            return Err(ParseError::InvalidMagic {
                expected: ZIP64_EOCD_LOCATOR_SIGNATURE.to_le_bytes().to_vec(),
                found: sig.to_le_bytes().to_vec(),
            });
        }

        let zip64_eocd_offset = u64::from_le_bytes([
            locator[8], locator[9], locator[10], locator[11],
            locator[12], locator[13], locator[14], locator[15],
        ]);

        // Parse ZIP64 EOCD
        reader.seek(SeekFrom::Start(zip64_eocd_offset))?;

        let mut zip64_eocd = [0u8; 56];
        reader.read_exact(&mut zip64_eocd)?;

        let sig = u32::from_le_bytes([zip64_eocd[0], zip64_eocd[1], zip64_eocd[2], zip64_eocd[3]]);
        if sig != ZIP64_EOCD_SIGNATURE {
            return Err(ParseError::InvalidMagic {
                expected: ZIP64_EOCD_SIGNATURE.to_le_bytes().to_vec(),
                found: sig.to_le_bytes().to_vec(),
            });
        }

        let total_entries = u64::from_le_bytes([
            zip64_eocd[32], zip64_eocd[33], zip64_eocd[34], zip64_eocd[35],
            zip64_eocd[36], zip64_eocd[37], zip64_eocd[38], zip64_eocd[39],
        ]);

        let cd_offset = u64::from_le_bytes([
            zip64_eocd[48], zip64_eocd[49], zip64_eocd[50], zip64_eocd[51],
            zip64_eocd[52], zip64_eocd[53], zip64_eocd[54], zip64_eocd[55],
        ]);

        Ok((cd_offset, total_entries))
    }

    /// Parse central directory entries
    fn parse_central_directory<R: Read + Seek>(
        &self,
        reader: &mut R,
        eocd: &EndOfCentralDirectory,
        progress: Option<&ProgressCallback>,
    ) -> ParseResult<Vec<P4kEntry>> {
        reader.seek(SeekFrom::Start(eocd.cd_offset))?;

        let mut entries = Vec::with_capacity(eocd.total_entries as usize);

        for i in 0..eocd.total_entries {
            let entry = self.parse_cd_entry(reader)?;
            entries.push(entry);

            if let Some(ref cb) = progress {
                if i % 1000 == 0 {
                    cb(ParseProgress {
                        phase: ParsePhase::Indexing,
                        bytes_processed: reader.stream_position()?,
                        total_bytes: Some(eocd.cd_offset + eocd.cd_size),
                        current_item: entries.last().map(|e| e.path.clone()),
                        items_processed: i,
                        total_items: Some(eocd.total_entries),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Parse a single central directory entry
    fn parse_cd_entry<R: Read + Seek>(&self, reader: &mut R) -> ParseResult<P4kEntry> {
        let mut header = [0u8; 46];
        reader.read_exact(&mut header)?;

        let sig = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        if sig != CD_SIGNATURE {
            return Err(ParseError::InvalidMagic {
                expected: CD_SIGNATURE.to_le_bytes().to_vec(),
                found: sig.to_le_bytes().to_vec(),
            });
        }

        let version_made        = u16::from_le_bytes([header[4], header[5]]);
        let version_needed      = u16::from_le_bytes([header[6], header[7]]);
        let flags               = u16::from_le_bytes([header[8], header[9]]);
        let compression         = CompressionMethod::from(u16::from_le_bytes([header[10], header[11]]));
        let mod_time            = u16::from_le_bytes([header[12], header[13]]);
        let mod_date            = u16::from_le_bytes([header[14], header[15]]);
        let crc32               = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
        let compressed_size     = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);
        let uncompressed_size   = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
        let name_length       = u16::from_le_bytes([header[28], header[29]]) as usize;
        let extra_length      = u16::from_le_bytes([header[30], header[31]]) as usize;
        let comment_length    = u16::from_le_bytes([header[32], header[33]]) as usize;
        let disk_start          = u16::from_le_bytes([header[34], header[35]]);
        let internal_attrs      = u16::from_le_bytes([header[36], header[37]]);
        let external_attrs      = u32::from_le_bytes([header[38], header[39], header[40], header[41]]);
        let local_header_offset = u32::from_le_bytes([header[42], header[43], header[44], header[45]]);

        // Read filename
        let mut name_bytes = vec![0u8; name_length];
        reader.read_exact(&mut name_bytes)?;
        let path = String::from_utf8_lossy(&name_bytes).to_string();

        // Read extra field
        let mut extra = vec![0u8; extra_length];
        reader.read_exact(&mut extra)?;

        // Parse ZIP64 extra field if present
        let (compressed_size, uncompressed_size, local_header_offset) = 
            self.parse_zip64_extra(&extra, compressed_size, uncompressed_size, local_header_offset)?;

        // Skip comment
        reader.seek(SeekFrom::Current(comment_length as i64))?;

        Ok(P4kEntry {
            path,
            compression,
            crc32,
            compressed_size,
            uncompressed_size,
            local_header_offset,
            flags,
            mod_time,
            mod_date,
            is_encrypted: flags & 0x01 != 0,
            is_directory: path.ends_with('/'),
        })
    }

    /// Parse ZIP64 extra field
    fn parse_zip64_extra(
        &self,
        extra: &[u8],
        compressed_size: u32,
        uncompressed_size: u32,
        local_offset: u32,
    ) -> ParseResult<(u64, u64, u64)> {
        let mut compressed      = compressed_size as u64;
        let mut uncompressed    = uncompressed_size as u64;
        let mut offset          = local_offset as u64;

        let mut pos = 0;
        while pos + 4 <= extra.len() {
            let id = u16::from_le_bytes([extra[pos], extra[pos + 1]]);
            let size = u16::from_le_bytes([extra[pos + 2], extra[pos + 3]]) as usize;
            pos += 4;

            if id == 0x0001 && pos + size <= extra.len() {
                // ZIP64 extra field
                let mut field_pos = 0;

                if uncompressed_size == 0xFFFFFFFF && field_pos + 8 <= size {
                    uncompressed = u64::from_le_bytes([
                        extra[pos + field_pos], extra[pos + field_pos + 1],
                        extra[pos + field_pos + 2], extra[pos + field_pos + 3],
                        extra[pos + field_pos + 4], extra[pos + field_pos + 5],
                        extra[pos + field_pos + 6], extra[pos + field_pos + 7],
                    ]);
                    field_pos += 8;
                }

                if compressed_size == 0xFFFFFFF && field_pos + 8 <= size {
                    compressed = u64::from_le_bytes([
                        extra[pos + field_pos], extra[pos + field_pos + 1],
                        extra[pos + field_pos + 2], extra[pos + field_pos + 3],
                        extra[pos + field_pos + 4], extra[pos + field_pos + 5],
                        extra[pos + field_pos + 6], extra[pos + field_pos + 7],
                    ]);
                    field_pos += 8;
                }

                if local_offset == 0xFFFFFFF && field_pos + 8 <= size {
                    offset = u64::from_le_bytes([
                        extra[pos + field_pos], extra[pos + field_pos + 1],
                        extra[pos + field_pos + 2], extra[pos + field_pos + 3],
                        extra[pos + field_pos + 4], extra[pos + field_pos + 5],
                        extra[pos + field_pos + 6], extra[pos + field_pos + 7],
                    ]);
                }

                break;
            }

            pos += size;
        }

        Ok((compressed, uncompressed, offset))
    }

    /// Extract file data from local header
    fn extract_data<R: Read + Seek>(
        &self,
        reader: &mut R,
        entry: &P4kEntry,
    ) -> ParseResult<Vec<u8>> {
        reader.seek(SeekFrom::Start(entry.local_header_offset))?;

        // Read local header
        let mut local_header = [0u8; 30];
        reader.read_exact(&mut local_header)?;

        let sig = u32::from_le_bytes([local_header[0], local_header[1], local_header[2], local_header[3]]);
        if sig != LOCAL_HEADER_SIGNATURE {
            return Err(ParseError::InvalidMagic {
                expected: LOCAL_HEADER_SIGNATURE.to_le_bytes().to_vec(),
                found: sig.to_le_bytes().to_vec(),
            });
        }

        let name_len = u16::from_le_bytes([local_header[26], local_header[27]]) as u64;
        let extra_len = u16::from_le_bytes([local_header[28], local_header[29]]) as u64;

        // Skip to data
        reader.seek(SeekFrom::Current((name_len + extra_len) as i64))?;

        // Read compressed data
        let mut compressed = vec![0u8; entry.compressed_size as usize];
        reader.read_exact(&mut compressed)?;

        // Decompress
        let decompressed = P4kCompression::decompress(
            &compressed,
            entry.compression,
            entry.uncompressed_size as usize,
        )?;

        Ok(decompressed)
    }
}

impl Default for P4kParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for P4kParser {
    type Output = P4kArchive;

    fn extensions(&self) -> &[&str] {
        &["p4k"]
    }

    fn magic_bytes(&self) -> Option<&[u8]> {
        Some(P4K_MAGIC)
    }

    fn name(&self) -> &str {
        "P4K Archive Parser"
    }

    fn parse_with_options<R: Read + Seek>(
        &self,
        mut reader: R,
        _options: &ParseOptions,
        progress: Option<ProgressCallback>,
    ) -> ParseResult<Self::Output> {
        // Verify magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != P4K_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: P4K_MAGIC.to_vec(),
                found: magic.to_vec(),
            });
        }

        // Report header reading progress
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::ReadingHeader,
                bytes_processed: 4,
                total_bytes: None,
                current_item: None,
                items_processed: 0,
                total_items: None,
            });
        }

        // Parse EOCD
        let eocd = self.parse_eocd(&mut reader)?;

        // Parse central directory
        let entries = self.parse_central_directory(&mut reader, &eocd, progress.as_ref())?;

        // Build path index
        let mut path_index = HashMap::with_capacity(entries.len());
        for (idx, entry) in entries.iter().enumerate() {
            path_index.insert(entry.path.clone(), idx);
        }

        // Report completion
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::Complete,
                bytes_processed: reader.stream_position()?,
                total_bytes: None,
                current_item: None,
                items_processed: entries.len() as u64,
                total_items: Some(entries.len() as u64),
            });
        }

        Ok(P4kArchive {
            entries,
            path_index,
        })
    }
}

impl RandomAccessParser for P4kParser {
    type EntryId = String;
    type EntryMeta = P4kEntryInfo;

    fn list_entries<R: Read + Seek>(&self, reader: R) -> ParseResult<Vec<(Self::EntryId, Self::EntryMeta)>> {
        let archive = self.parse(reader)?;

        Ok(archive.entries.iter().map(|e| {
            (e.path.clone(), P4kEntryInfo {
                path: e.path.clone(),
                compressed_size: e.compressed_size,
                uncompressed_size: e.uncompressed_size,
                is_directory: e.is_directory,
                compression: e.compression,
            })
        }).collect())
    }

    fn extract_entry<R: Read + Seek>(
        &self,
        mut reader: R,
        entry_id: &Self::EntryId,
    ) -> ParseResult<Vec<u8>> {
        let archive = self.parse(&mut reader)?;

        let idx = archive.path_index.get(entry_id)
            .ok_or_else(|| ParseError::MissingField(format!("Entry not found: {}", entry_id)))?;

        let entry = &archive.entries[*idx];
        self.extract_data(&mut reader, entry)
    }
}

/// End of Central Directory record
#[derive(Debug)]
struct EndOfCentralDirectory {
    disk_number: u16,
    cd_disk: u16,
    disk_entries: u64,
    total_entries: u64,
    cd_size: u64,
    cd_offset: u64,
    comment_length: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_method_conversion() {
        assert_eq!(CompressionMethod::from(0), CompressionMethod::Store);
        assert_eq!(CompressionMethod::from(8), CompressionMethod::Deflate);
        assert_eq!(CompressionMethod::from(93), CompressionMethod::Zstd);
        assert_eq!(CompressionMethod::from(99), CompressionMethod::Lz4);
        assert_eq!(CompressionMethod::from(255), CompressionMethod::Unknown(255));
    }
}