use anyhow::{Ok, Result};
use log::info;
use std::{fs::File, io::{Read, Seek, SeekFrom}};

use crate::chunk::Chunk;

#[derive(Debug)]
pub struct Document<R: Read + Seek> {
    reader: R,
    chunks: Vec<Chunk>,
    document_size: usize,
    chunk_size: usize,
}

const DEFAULT_CHUNK_SIZE: usize = 65536;

impl<R: Read + Seek> Document<R> {

    fn new(mut reader: R, chunk_size: usize) -> Result<Self> {  
        let document_size = reader.seek(SeekFrom::End(0))? as usize;
        Ok(Self { reader, chunks: vec![], document_size, chunk_size })
    }  

    pub fn open_file(filename: &str) -> Result<Document<File>> {
        let file = File::open(filename)?;
        Document::<File>::new(file, DEFAULT_CHUNK_SIZE)
    }

    fn load_chunk(&mut self, offset_begin: usize) -> Result<()> {
        info!("[Document::load_chunk] offset: {offset_begin}");
        assert!(offset_begin < self.document_size);
        let mut buffer = vec![0; self.chunk_size];
        self.reader.seek(SeekFrom::Start(offset_begin as u64))?;
        let consumed = self.reader.read(&mut buffer)?;
        let content = std::str::from_utf8(&buffer[..consumed])?;
        self.chunks.push(Chunk::build_chunk(content, offset_begin, false, true));
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, vec};

    #[test]
    fn test_load_chunk() {
        let cursor = Cursor::new("1234\nabcd\n9876\nqwer\n0011\n7788\nccvv\nzxcv");
        let mut doc = Document::new(cursor.clone(), 6).unwrap();
        doc.load_chunk(0).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 5);
        assert_eq!(doc.chunks[0].rows, vec!["1234"]);

        doc.load_chunk(5).unwrap();
        assert_eq!(doc.chunks[1].offset_begin, 5);
        assert_eq!(doc.chunks[1].offset_end, 10);
        assert_eq!(doc.chunks[1].rows, vec!["abcd"]);

        let mut doc = Document::new(cursor.clone(), 29).unwrap();
        doc.load_chunk(0).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 25);
        assert_eq!(doc.chunks[0].rows, vec!["1234", "abcd", "9876", "qwer", "0011"]);

        let mut doc = Document::new(cursor.clone(), 30).unwrap();
        doc.load_chunk(0).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 30);
        assert_eq!(doc.chunks[0].rows, vec!["1234", "abcd", "9876", "qwer", "0011", "7788"]);
    }

}