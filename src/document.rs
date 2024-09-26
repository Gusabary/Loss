use anyhow::{Ok, Result};
use log::info;
use std::{fs::File, io::{Read, Seek, SeekFrom}};

use crate::chunk::{self, Chunk};

#[derive(Debug)]
pub struct Document<R: Read + Seek> {
    reader: R,
    chunks: Vec<Chunk>,
    document_size: usize,
    default_chunk_size: usize,
}

const DEFAULT_CHUNK_SIZE: usize = 65536;

impl<R: Read + Seek> Document<R> {

    fn new(mut reader: R) -> Result<Self> {  
        let document_size = reader.seek(SeekFrom::End(0))? as usize;
        Ok(Self { reader, chunks: vec![], document_size, default_chunk_size: DEFAULT_CHUNK_SIZE })
    }  

    pub fn open_file(filename: &str) -> Result<Document<File>> {
        let file = File::open(filename)?;
        Document::<File>::new(file)
    }

    fn load_chunk(&mut self, mut offset_begin: usize, mut offset_end: usize) -> Result<Option<usize>> {
        assert!(offset_begin < offset_end);
        assert!(offset_end <= self.document_size);

        // avoid chunk overlap
        if let Some(chunk_index_begin) = self.get_chunk_index_by_offset(offset_begin) {
            for index in chunk_index_begin..self.chunks.len() {
                if offset_begin < self.chunks[index].offset_begin {
                    break;
                }
                offset_begin = self.chunks[index].offset_end;
            }
        }
        if let Some(chunk_index_end) = self.get_chunk_index_by_offset(offset_end) {
            for index in (0..=chunk_index_end).rev() {
                if offset_end > self.chunks[index].offset_end {
                    break;
                }
                offset_end = self.chunks[index].offset_begin;
            }
        }
        if offset_begin >= offset_end {
            return Ok(None);
        }
        if offset_begin > 0 {
            // actually a temporary hack to make sure first line is not dropped
            offset_begin -= 1;
        }
        
        // build chunk
        let mut buffer = vec![0; offset_end - offset_begin];
        self.reader.seek(SeekFrom::Start(offset_begin as u64))?;
        let consumed = self.reader.read(&mut buffer)?;
        let content = std::str::from_utf8(&buffer[..consumed])?;
        // drop first unless loading chunk starting from the first byte
        let drop_first = offset_begin > 0;  
        let new_chunk = Chunk::build_chunk(content, offset_begin, drop_first, true);

        // add into chunk list
        let mut new_chunk_index = 0;
        while new_chunk_index < self.chunks.len() {
            if self.chunks[new_chunk_index].offset_begin >= new_chunk.offset_begin {
                break;
            }
            new_chunk_index += 1;
        }
        let mut remove_until_index = new_chunk_index;
        for index in new_chunk_index..self.chunks.len() {
            if self.chunks[index].offset_end <= new_chunk.offset_end {
                remove_until_index = index + 1;
                continue;
            }
        }
        self.chunks.drain(new_chunk_index..remove_until_index);
        self.chunks.insert(new_chunk_index, new_chunk);
        Ok(Some(new_chunk_index))
    }

    fn load_chunk_around(&mut self, offset: usize) -> Result<Option<usize>> {
        let offset_begin = offset.saturating_sub(self.default_chunk_size / 2);
        let offset_end = offset.saturating_add(self.default_chunk_size / 2);
        self.load_chunk(offset_begin, offset_end)
    }

    fn get_chunk_index_by_offset(&self, offset: usize) -> Option<usize> {
        for (index, chunk) in self.chunks.iter().enumerate() {
            if offset >= chunk.offset_end {
                continue;
            }
            if offset >= chunk.offset_begin {
                return Some(index);
            }
            if offset < chunk.offset_begin {
                return None
            }
        }
        None
    }

    // pub fn query_lines(&mut self, offset: usize, line_count: usize) -> Vec<String> {

    // }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, vec};

    #[test]
    fn test_get_chunk_index_by_offset() {
        let cursor = Cursor::new("");
        let mut doc = Document::new(cursor).unwrap();
        doc.chunks.push(Chunk {offset_begin: 0, offset_end: 5, rows: vec![]});
        doc.chunks.push(Chunk {offset_begin: 5, offset_end: 10, rows: vec![]});
        doc.chunks.push(Chunk {offset_begin: 15, offset_end: 20, rows: vec![]});
        assert_eq!(doc.get_chunk_index_by_offset(0), Some(0));
        assert_eq!(doc.get_chunk_index_by_offset(2), Some(0));
        assert_eq!(doc.get_chunk_index_by_offset(5), Some(1));
        assert_eq!(doc.get_chunk_index_by_offset(10), None);
        assert_eq!(doc.get_chunk_index_by_offset(15), Some(2));
        assert_eq!(doc.get_chunk_index_by_offset(17), Some(2));
        assert_eq!(doc.get_chunk_index_by_offset(21), None);
    }

    #[test]
    fn test_load_chunk() {
        let cursor = Cursor::new("1234\n1234\n1234\n1234\n1234\n1234\n1234\n1234\n");
        let mut doc = Document::new(cursor.clone()).unwrap();
        doc.load_chunk(0, 11).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 10);
        assert_eq!(doc.chunks[0].rows, vec!["1234", "1234"]);

        doc.load_chunk(5, 16).unwrap();
        assert_eq!(doc.chunks[1].offset_begin, 10);
        assert_eq!(doc.chunks[1].offset_end, 15);
        assert_eq!(doc.chunks[1].rows.len(), 1);

        doc.load_chunk(28, 39).unwrap();
        assert_eq!(doc.chunks[2].offset_begin, 30);
        assert_eq!(doc.chunks[2].offset_end, 35);
        assert_eq!(doc.chunks[2].rows.len(), 1);

        doc.load_chunk(15, 28).unwrap();
        assert_eq!(doc.chunks[2].offset_begin, 15);
        assert_eq!(doc.chunks[2].offset_end, 25);
        assert_eq!(doc.chunks[3].offset_begin, 30);
        assert_eq!(doc.chunks[3].offset_end, 35);

        doc.load_chunk(18, 32).unwrap();
        assert_eq!(doc.chunks[3].offset_begin, 25);
        assert_eq!(doc.chunks[3].offset_end, 30);
        assert_eq!(doc.chunks[4].offset_begin, 30);
        assert_eq!(doc.chunks[4].offset_end, 35);

        doc.load_chunk(6, 31).unwrap();
        assert_eq!(doc.chunks.len(), 5);
    }

    #[test]
    fn test_load_chunk_drain() {
        let cursor = Cursor::new("1234\n1234\n1234\n1234\n1234\n1234\n1234\n1234\n");
        let mut doc = Document::new(cursor.clone()).unwrap();
        doc.load_chunk(0, 11).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 10);

        doc.load_chunk(15, 21).unwrap();
        assert_eq!(doc.chunks[1].offset_begin, 15);
        assert_eq!(doc.chunks[1].offset_end, 20);

        doc.load_chunk(23, 32).unwrap();
        assert_eq!(doc.chunks[2].offset_begin, 25);
        assert_eq!(doc.chunks[2].offset_end, 30);

        doc.load_chunk(35, 40).unwrap();
        assert_eq!(doc.chunks[3].offset_begin, 35);
        assert_eq!(doc.chunks[3].offset_end, 40);

        doc.load_chunk(12, 32).unwrap();
        assert_eq!(doc.chunks.len(), 3);
        assert_eq!(doc.chunks[1].offset_begin, 15);
        assert_eq!(doc.chunks[1].offset_end, 30);
        assert_eq!(doc.chunks[2].offset_begin, 35);
        assert_eq!(doc.chunks[2].offset_end, 40);
    }

}