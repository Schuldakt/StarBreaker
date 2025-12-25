//! File streaming utilities for large files

use std::io::{Read, Result as IoResult};

/// Buffered reader for VFS files
/// Provides efficient streaming of large files with configurable buffer size
pub struct VfsStreamReader {
    inner: Box<dyn Read + Send>,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
}

impl VfsStreamReader {
    /// Create a new stream reader with default buffer size (64KB)
    pub fn new(reader: Box<dyn Read + Send>) -> Self {
        Self::with_capacity(reader, 64 * 1024)
    }

    /// Create a new stream reader with custom buffer size
    pub fn with_capacity(reader: Box<dyn Read + Send>, capacity: usize) -> Self {
        Self {
            inner: reader,
            buffer: vec![0; capacity],
            buffer_pos: 0,
            buffer_len: 0,
        }
    }

    /// Read data into internal buffer
    fn fill_buffer(&mut self) -> IoResult<usize> {
        self.buffer_len = self.inner.read(&mut self.buffer)?;
        self.buffer_pos = 0;
        Ok(self.buffer_len)
    }

    /// Read exactly n bytes or return error
    pub fn read_exact_bytes(&mut self, n: usize) -> IoResult<Vec<u8>> {
        let mut result = vec![0; n];
        self.read_exact(&mut result)?;
        Ok(result)
    }

    /// Skip n bytes in the stream
    pub fn skip(&mut self, mut n: usize) -> IoResult<()> {
        while n > 0 {
            let available = self.buffer_len - self.buffer_pos;
            
            if available > 0 {
                let to_skip = n.min(available);
                self.buffer_pos += to_skip;
                n -= to_skip;
            } else {
                // Buffer empty, refill
                if self.fill_buffer()? == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "End of stream while skipping"
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Peek at next n bytes without consuming
    pub fn peek(&mut self, n: usize) -> IoResult<&[u8]> {
        let available = self.buffer_len - self.buffer_pos;
        
        if available >= n {
            Ok(&self.buffer[self.buffer_pos..self.buffer_pos + n])
        } else {
            // Need to refill buffer
            if self.buffer_pos > 0 {
                // Move remaining data to start
                self.buffer.copy_within(self.buffer_pos..self.buffer_len, 0);
                self.buffer_len -= self.buffer_pos;
                self.buffer_pos = 0;
            }
            
            // Read more data
            let read = self.inner.read(&mut self.buffer[self.buffer_len..])?;
            self.buffer_len += read;
            
            if self.buffer_len >= n {
                Ok(&self.buffer[0..n])
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Not enough data to peek"
                ))
            }
        }
    }
}

impl Read for VfsStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let available = self.buffer_len - self.buffer_pos;
        
        if available > 0 {
            // We have buffered data
            let to_copy = available.min(buf.len());
            buf[..to_copy].copy_from_slice(
                &self.buffer[self.buffer_pos..self.buffer_pos + to_copy]
            );
            self.buffer_pos += to_copy;
            Ok(to_copy)
        } else {
            // Buffer empty - for large reads, bypass buffer
            if buf.len() >= self.buffer.len() {
                self.inner.read(buf)
            } else {
                // Small read - refill buffer first
                self.fill_buffer()?;
                self.read(buf)
            }
        }
    }
}

/// Chunked reader for processing large files in fixed-size chunks
pub struct ChunkedReader {
    reader: VfsStreamReader,
    chunk_size: usize,
}

impl ChunkedReader {
    /// Create a new chunked reader
    pub fn new(reader: Box<dyn Read + Send>, chunk_size: usize) -> Self {
        Self {
            reader: VfsStreamReader::new(reader),
            chunk_size,
        }
    }

    /// Read next chunk
    /// Returns None if end of stream reached
    pub fn read_chunk(&mut self) -> IoResult<Option<Vec<u8>>> {
        let mut chunk = vec![0; self.chunk_size];
        let mut total_read = 0;

        loop {
            match self.reader.read(&mut chunk[total_read..]) {
                Ok(0) => {
                    // End of stream
                    if total_read == 0 {
                        return Ok(None);
                    } else {
                        chunk.truncate(total_read);
                        return Ok(Some(chunk));
                    }
                }
                Ok(n) => {
                    total_read += n;
                    if total_read >= self.chunk_size {
                        return Ok(Some(chunk));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// Process file in chunks with a callback
    pub fn process_chunks<F>(&mut self, mut callback: F) -> IoResult<()>
    where
        F: FnMut(&[u8]) -> IoResult<()>,
    {
        while let Some(chunk) = self.read_chunk()? {
            callback(&chunk)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_stream_reader() {
        let data = b"Hello, World!";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = VfsStreamReader::new(Box::new(cursor));

        let mut buf = [0u8; 5];
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"Hello");
    }

    #[test]
    fn test_skip() {
        let data = b"0123456789";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = VfsStreamReader::new(Box::new(cursor));

        reader.skip(5).unwrap();
        let mut buf = [0u8; 5];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"56789");
    }

    #[test]
    fn test_chunked_reader() {
        let data = b"ABCDEFGHIJKLMNOP";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ChunkedReader::new(Box::new(cursor), 4);

        let chunk1 = reader.read_chunk().unwrap().unwrap();
        assert_eq!(&chunk1, b"ABCD");

        let chunk2 = reader.read_chunk().unwrap().unwrap();
        assert_eq!(&chunk2, b"EFGH");
    }
}
