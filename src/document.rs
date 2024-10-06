use anyhow::{Ok, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use log::info;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::chunk::Chunk;
use crate::log_timestamp::detect_log_timstamp_format;

#[derive(Debug)]
pub struct Document<R: Read + Seek> {
    reader: R,
    chunks: Vec<Chunk>,
    log_timestamp_format: Option<String>,
    log_default_date: Option<NaiveDate>,
    last_line: Option<String>,
    document_size: usize,
    default_chunk_size: usize,
}

const DEFAULT_CHUNK_SIZE: usize = 65536;

impl<R: Read + Seek> Document<R> {
    fn new(mut reader: R) -> Result<Self> {
        let document_size = reader.seek(SeekFrom::End(0))? as usize;
        let mut document = Self {
            reader,
            chunks: vec![],
            log_timestamp_format: None,
            log_default_date: None,
            last_line: None,
            document_size,
            default_chunk_size: DEFAULT_CHUNK_SIZE,
        };
        if document_size > 0 {
            document.load_chunk(
                document_size.saturating_sub(DEFAULT_CHUNK_SIZE),
                document_size,
            )?;
            assert!(document.last_line.is_some());
        } else {
            document.last_line = Some(String::default());
        }
        Ok(document)
    }

    pub fn open_file(filename: &str) -> Result<Document<File>> {
        let file = File::open(filename)?;
        Document::<File>::new(file)
    }

    pub fn last_line_start_offset(&self) -> usize {
        assert!(self.last_line.is_some());
        self.document_size - self.last_line.as_ref().unwrap().len()
    }

    pub fn percent_ratio_of_offset(&self, offset: usize) -> usize {
        if self.last_line_start_offset() == 0 {
            100
        } else {
            offset * 100 / self.last_line_start_offset()
        }
    }

    fn load_chunk(
        &mut self,
        mut offset_begin: usize,
        mut offset_end: usize,
    ) -> Result<Option<usize>> {
        info!("[load_chunk] offset_begin: {offset_begin} offset_end: {offset_end}");
        offset_end = std::cmp::min(offset_end, self.document_size);
        assert!(offset_begin < offset_end);

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
        // actually a temporary hack to make sure first line is not dropped
        offset_begin = offset_begin.saturating_sub(1);

        // build chunk
        let mut buffer = vec![0; offset_end - offset_begin];
        self.reader.seek(SeekFrom::Start(offset_begin as u64))?;
        let consumed = self.reader.read(&mut buffer)?;
        assert!(consumed > 0, "cannot read anything from file");
        let content = std::str::from_utf8(&buffer[..consumed])?;
        // drop first unless loading chunk starting from the first byte
        let drop_first = offset_begin > 0;
        let cover_end = offset_end >= self.document_size;
        let mut new_chunk = Chunk::build_chunk(content, offset_begin, drop_first, !cover_end);

        if cover_end {
            // handle last line
            assert!(!new_chunk.rows.is_empty());
            let mut last_line = new_chunk.rows.pop().unwrap();
            if content.ends_with('\n') {
                last_line.push('\n');
            }
            new_chunk.offset_end -= last_line.len();
            self.last_line = Some(last_line);
        }
        if new_chunk.rows.is_empty() {
            return Ok(None);
        }

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
        info!("[load_chunk_around] offset: {offset}");
        let offset_begin = offset.saturating_sub(self.default_chunk_size / 2);
        let offset_end = offset.saturating_add(self.default_chunk_size / 2);
        self.load_chunk(offset_begin, offset_end)
    }

    fn get_chunk_index_by_offset(&self, offset: usize) -> Option<usize> {
        info!("[get_chunk_index_by_offset] offset: {offset}");
        for (index, chunk) in self.chunks.iter().enumerate() {
            if offset >= chunk.offset_end {
                continue;
            }
            if offset >= chunk.offset_begin {
                return Some(index);
            }
            if offset < chunk.offset_begin {
                return None;
            }
        }
        None
    }

    fn get_or_load_chunk_by_offset(&mut self, offset: usize) -> Result<&Chunk> {
        info!("[get_or_load_chunk_by_offset] offset: {offset}");
        let chunk_index_opt = self.get_chunk_index_by_offset(offset);
        let chunk_index = if let Some(chunk_index) = chunk_index_opt {
            chunk_index
        } else {
            self.load_chunk_around(offset)?.unwrap()
        };
        let chunk = &self.chunks[chunk_index];
        Ok(chunk)
    }

    pub fn query_lines(&mut self, mut offset: usize, mut line_count: usize) -> Result<Vec<String>> {
        info!("[query_lines] offset: {offset} line_count: {line_count}");
        let mut lines: Vec<String> = vec![];
        while offset < self.last_line_start_offset() && line_count > 0 {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let line_index = chunk.query_line_index_exactly(offset);
            let line_count_taken = std::cmp::min(line_count, chunk.rows.len() - line_index);
            lines.extend(
                chunk
                    .rows
                    .iter()
                    .skip(line_index)
                    .take(line_count_taken)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            line_count -= line_count_taken;
            offset = chunk.offset_end;
        }
        if line_count > 0 {
            lines.push(self.last_line_without_line_break());
        }
        Ok(lines)
    }

    fn last_line_without_line_break(&self) -> String {
        let mut last_line = self.last_line.clone().unwrap();
        if last_line.ends_with('\n') {
            last_line.pop();
        }
        last_line
    }

    pub fn query_distance_to_above_n_lines(
        &mut self,
        mut offset: usize,
        mut line_count: usize,
    ) -> Result<usize> {
        info!("[query_distance_to_above_n_lines] offset: {offset} line_count: {line_count}");
        // offset must be at the line start
        let mut distance = 0;
        let mut first_loop = true;
        assert!(offset <= self.last_line_start_offset());
        if offset == self.last_line_start_offset() {
            offset = offset.saturating_sub(1);
            first_loop = false;
        }
        while offset > 0 && line_count > 0 {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let above_lines_in_chunk = if first_loop {
                chunk.query_line_index_exactly(offset)
            } else {
                chunk.query_line_index(offset) + 1
            };
            let line_count_skipped = chunk.rows.len() - above_lines_in_chunk;
            let line_count_taken = std::cmp::min(line_count, above_lines_in_chunk);

            distance += chunk
                .rows
                .iter()
                .rev()
                .skip(line_count_skipped)
                .take(line_count_taken)
                // count in the \n
                .map(|line| line.len() + 1)
                .sum::<usize>();
            line_count -= line_count_taken;
            offset = chunk.offset_begin.saturating_sub(1);
            first_loop = false;
        }
        Ok(distance)
    }

    pub fn query_distance_to_below_n_lines(
        &mut self,
        mut offset: usize,
        mut line_count: usize,
    ) -> Result<usize> {
        info!("[query_distance_to_below_n_lines] offset: {offset} line_count: {line_count}");
        // offset must be at the line start
        let mut distance = 0;
        while offset < self.last_line_start_offset() && line_count > 0 {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let line_index = chunk.query_line_index_exactly(offset);
            let line_count_taken = std::cmp::min(line_count, chunk.rows.len() - line_index);
            distance += chunk
                .rows
                .iter()
                .skip(line_index)
                .take(line_count_taken)
                // count in the \n
                .map(|line| line.len() + 1)
                .sum::<usize>();
            line_count -= line_count_taken;
            offset = chunk.offset_end;
        }
        Ok(distance)
    }

    pub fn query_distance_to_prev_match(
        &mut self,
        mut offset: usize,
        search_pattern: &str,
    ) -> Result<Option<usize>> {
        // offset must be at the line start
        let mut distance = 0;
        let mut first_loop = true;
        assert!(offset <= self.last_line_start_offset());
        if offset == self.last_line_start_offset() {
            offset = offset.saturating_sub(1);
            first_loop = false;
        }
        while offset > 0 {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let above_lines_in_chunk = if first_loop {
                chunk.query_line_index_exactly(offset)
            } else {
                chunk.query_line_index(offset) + 1
            };
            let line_count_skipped = chunk.rows.len() - above_lines_in_chunk;
            for line in chunk.rows.iter().rev().skip(line_count_skipped) {
                distance += line.len() + 1;
                if line.contains(search_pattern) {
                    return Ok(Some(distance));
                }
            }
            offset = chunk.offset_begin.saturating_sub(1);
            first_loop = false;
        }
        Ok(None)
    }

    pub fn query_distance_to_next_match(
        &mut self,
        mut offset: usize,
        search_pattern: &str,
    ) -> Result<Option<usize>> {
        let mut distance = 0;
        while offset < self.last_line_start_offset() {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let line_index = chunk.query_line_index_exactly(offset);
            for line in chunk.rows.iter().skip(line_index) {
                if line.contains(search_pattern) {
                    return Ok(Some(distance));
                }
                distance += line.len() + 1;
            }
            offset = chunk.offset_end;
        }
        if self.last_line.as_ref().unwrap().contains(search_pattern) {
            Ok(Some(distance))
        } else {
            Ok(None)
        }
    }

    pub fn query_offset_by_timestamp(
        &mut self,
        date: Option<NaiveDate>,
        time: NaiveTime,
    ) -> Result<Option<usize>> {
        if self.chunks.is_empty() {
            // empty file or only single line
            return Ok(Some(0));
        }
        if self.log_timestamp_format.is_none() {
            self.load_log_timestamp_format_and_default_date();
        }
        if self.log_timestamp_format.is_none() {
            // cannot detect log timestamp format or default date
            return Ok(None);
        }
        let date = date.unwrap_or(self.log_default_date.unwrap());
        let target_datetime = NaiveDateTime::new(date, time);

        let mut offset_begin = 0;
        let mut offset_end = self.last_line_start_offset();
        let timestamp_format = self.log_timestamp_format.clone().unwrap();
        let mut offset = (offset_begin + offset_end) / 2;
        loop {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            let index = chunk.query_line_index(offset);
            if let Result::Ok((datetime, _)) =
                NaiveDateTime::parse_and_remainder(&chunk.rows[index], &timestamp_format)
            {
                if datetime >= target_datetime {
                    offset_end = offset;
                } else {
                    offset_begin = offset;
                }
                if offset_begin + DEFAULT_CHUNK_SIZE >= offset_end {
                    return Ok(Some(self.linear_search_timestamp(
                        offset_begin,
                        offset_end,
                        target_datetime,
                    )?));
                }
                offset = (offset_begin + offset_end) / 2;
            } else {
                // a log line without timestamp, try next line
                offset = chunk.query_line_start_offset(index + 1);
                if offset >= offset_end {
                    return Ok(None);
                }
            }
        }
    }

    fn load_log_timestamp_format_and_default_date(&mut self) {
        assert!(self.log_timestamp_format.is_none() && self.log_default_date.is_none());
        assert!(!self.chunks.is_empty());
        const SAMPLE_LINE_COUNT: usize = 100;
        for line in self.chunks[0].rows.iter().take(SAMPLE_LINE_COUNT) {
            if let Some(fmt) = detect_log_timstamp_format(line) {
                self.log_timestamp_format = Some(fmt.clone());
                self.log_default_date = Some(
                    NaiveDateTime::parse_and_remainder(line, &fmt)
                        .unwrap()
                        .0
                        .date(),
                );
                return;
            }
        }
    }

    fn linear_search_timestamp(
        &mut self,
        offset_begin: usize,
        offset_end: usize,
        target: NaiveDateTime,
    ) -> Result<usize> {
        let timestamp_format = self.log_timestamp_format.clone().unwrap();
        let mut offset = offset_begin;
        while offset < offset_end {
            let chunk = self.get_or_load_chunk_by_offset(offset)?;
            offset = chunk.offset_begin;
            for line in chunk.rows.iter() {
                if let Result::Ok((datetime, _)) =
                    NaiveDateTime::parse_and_remainder(line, &timestamp_format)
                {
                    if datetime >= target {
                        return Ok(offset);
                    }
                }
                offset += line.len() + 1;
            }
            assert_eq!(offset, chunk.offset_end);
        }
        Ok(offset_end)
    }

    pub fn assert_offset_is_at_line_start(&mut self, offset: usize) -> Result<()> {
        let chunk = self.get_or_load_chunk_by_offset(offset)?;
        chunk.query_line_index_exactly(offset);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, vec};

    #[test]
    fn test_query_distance_to_prev_match() {
        let cursor =
            Cursor::new("1234\nabcd\n1234\nabcd\n1234\nabcd\n1234\nabcd\n\n\n1234\nremain");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.query_distance_to_prev_match(0, "123").unwrap(), None);
        assert_eq!(doc.query_distance_to_prev_match(5, "123").unwrap(), Some(5));
        assert_eq!(
            doc.query_distance_to_prev_match(10, "123").unwrap(),
            Some(10)
        );
        assert_eq!(doc.query_distance_to_prev_match(0, "bcd").unwrap(), None);
        assert_eq!(doc.query_distance_to_prev_match(35, "34").unwrap(), Some(5));
        assert_eq!(doc.query_distance_to_prev_match(40, "bc").unwrap(), Some(5));
        assert_eq!(
            doc.query_distance_to_prev_match(47, "bc").unwrap(),
            Some(12)
        );
        assert_eq!(
            doc.query_distance_to_prev_match(47, "remain").unwrap(),
            None
        );
    }

    #[test]
    fn test_query_distance_to_next_match() {
        let cursor =
            Cursor::new("1234\nabcd\n1234\nabcd\n1234\nabcd\n1234\nabcd\n\n\n1234\nremain");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.query_distance_to_next_match(0, "123").unwrap(), Some(0));
        assert_eq!(doc.query_distance_to_next_match(5, "123").unwrap(), Some(5));
        assert_eq!(
            doc.query_distance_to_next_match(10, "123").unwrap(),
            Some(0)
        );
        assert_eq!(doc.query_distance_to_next_match(0, "bcd").unwrap(), Some(5));
        assert_eq!(doc.query_distance_to_next_match(35, "34").unwrap(), Some(7));
        assert_eq!(doc.query_distance_to_next_match(35, "abcde").unwrap(), None);
        assert_eq!(
            doc.query_distance_to_next_match(35, "main").unwrap(),
            Some(12)
        );
        assert_eq!(
            doc.query_distance_to_next_match(47, "main").unwrap(),
            Some(0)
        );
    }

    #[test]
    fn test_query_distance_to_above_n_lines() {
        let cursor =
            Cursor::new("1234\nabcd\n1234\nabcd\n1234\nabcd\n1234\nabcd\n\n\n1234\nremain");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.query_distance_to_above_n_lines(0, 0).unwrap(), 0);
        assert_eq!(doc.query_distance_to_above_n_lines(5, 0).unwrap(), 0);
        assert_eq!(doc.query_distance_to_above_n_lines(5, 1).unwrap(), 5);
        assert_eq!(doc.query_distance_to_above_n_lines(20, 1).unwrap(), 5);
        assert_eq!(doc.query_distance_to_above_n_lines(30, 6).unwrap(), 30);
        assert_eq!(doc.query_distance_to_above_n_lines(35, 7).unwrap(), 35);
        assert_eq!(doc.query_distance_to_above_n_lines(35, 10).unwrap(), 35);
        assert_eq!(doc.query_distance_to_above_n_lines(40, 0).unwrap(), 0);
        assert_eq!(doc.query_distance_to_above_n_lines(40, 1).unwrap(), 5);
        assert_eq!(doc.query_distance_to_above_n_lines(40, 2).unwrap(), 10);
        assert_eq!(doc.query_distance_to_above_n_lines(41, 1).unwrap(), 1);
        assert_eq!(doc.query_distance_to_above_n_lines(41, 2).unwrap(), 6);
        assert_eq!(doc.query_distance_to_above_n_lines(41, 3).unwrap(), 11);
        assert_eq!(doc.query_distance_to_above_n_lines(42, 1).unwrap(), 1);
        assert_eq!(doc.query_distance_to_above_n_lines(42, 2).unwrap(), 2);
        assert_eq!(doc.query_distance_to_above_n_lines(42, 3).unwrap(), 7);
        assert_eq!(doc.query_distance_to_above_n_lines(47, 0).unwrap(), 0);
        assert_eq!(doc.query_distance_to_above_n_lines(47, 1).unwrap(), 5);
        assert_eq!(doc.query_distance_to_above_n_lines(47, 2).unwrap(), 6);
        assert_eq!(doc.query_distance_to_above_n_lines(47, 3).unwrap(), 7);
        assert_eq!(doc.query_distance_to_above_n_lines(47, 4).unwrap(), 12);
    }

    #[test]
    fn test_query_distance_to_below_n_lines() {
        let cursor = Cursor::new("1234\nabcd\n1234\nabcd\n1234\nabcd\n1234\nabcd\nremain");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.query_distance_to_below_n_lines(0, 2).unwrap(), 10);
        assert_eq!(doc.query_distance_to_below_n_lines(0, 6).unwrap(), 30);
        assert_eq!(doc.query_distance_to_below_n_lines(5, 0).unwrap(), 0);
        assert_eq!(doc.query_distance_to_below_n_lines(20, 1).unwrap(), 5);
        assert_eq!(doc.query_distance_to_below_n_lines(30, 6).unwrap(), 10);
    }

    #[test]
    fn test_query_lines() {
        let cursor = Cursor::new("1234\nabcd\n1234\nabcd\n1234\nabcd\n1234\nabcd\nremain");
        let mut doc = Document::new(cursor.clone()).unwrap();
        doc.default_chunk_size = 10;
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.last_line.as_ref().unwrap(), "remain");
        doc.chunks.pop();

        assert_eq!(doc.query_lines(0, 2).unwrap(), vec!["1234", "abcd"]);
        assert_eq!(doc.query_lines(15, 1).unwrap(), vec!["abcd"]);
        assert_eq!(doc.query_lines(0, 1).unwrap(), vec!["1234"]);
        assert_eq!(
            doc.query_lines(15, 3).unwrap(),
            vec!["abcd", "1234", "abcd"]
        );
        assert_eq!(doc.query_lines(35, 1).unwrap(), vec!["abcd"]);
        assert_eq!(doc.query_lines(35, 2).unwrap(), vec!["abcd", "remain"]);

        let cursor = Cursor::new("123456789\n\n\nabcd\n123456789\n");
        let mut doc = Document::new(cursor.clone()).unwrap();
        doc.default_chunk_size = 24;
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.last_line.as_ref().unwrap(), "123456789\n");
        doc.chunks.pop();

        assert_eq!(doc.query_lines(0, 2).unwrap(), vec!["123456789", ""]);
        assert_eq!(doc.query_lines(0, 3).unwrap(), vec!["123456789", "", ""]);
        assert_eq!(
            doc.query_lines(0, 4).unwrap(),
            vec!["123456789", "", "", "abcd"]
        );
        assert_eq!(
            doc.query_lines(0, 5).unwrap(),
            vec!["123456789", "", "", "abcd", "123456789"]
        );
        assert_eq!(
            doc.query_lines(0, 6).unwrap(),
            vec!["123456789", "", "", "abcd", "123456789"]
        );
        assert_eq!(
            doc.query_lines(10, 6).unwrap(),
            vec!["", "", "abcd", "123456789"]
        );
        assert_eq!(
            doc.query_lines(11, 6).unwrap(),
            vec!["", "abcd", "123456789"]
        );
        assert_eq!(doc.query_lines(12, 6).unwrap(), vec!["abcd", "123456789"]);
        assert_eq!(doc.query_lines(12, 1).unwrap(), vec!["abcd"]);

        assert_eq!(
            doc.chunks,
            vec![
                Chunk {
                    offset_begin: 0,
                    offset_end: 12,
                    rows: vec!["123456789".to_string(), "".to_string(), "".to_string()]
                },
                Chunk {
                    offset_begin: 12,
                    offset_end: 17,
                    rows: vec!["abcd".to_string()]
                },
            ]
        );
    }

    #[test]
    fn test_get_chunk_index_by_offset() {
        let cursor = Cursor::new("");
        let mut doc = Document::new(cursor).unwrap();
        doc.chunks.push(Chunk {
            offset_begin: 0,
            offset_end: 5,
            rows: vec![],
        });
        doc.chunks.push(Chunk {
            offset_begin: 5,
            offset_end: 10,
            rows: vec![],
        });
        doc.chunks.push(Chunk {
            offset_begin: 15,
            offset_end: 20,
            rows: vec![],
        });
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
        let cursor = Cursor::new("1234\n1234\n1234\n1234\n1234\n1234\n1234\n1234\nabc");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.last_line.as_ref().unwrap(), "abc");
        doc.chunks.pop();

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

        doc.load_chunk(30, 42).unwrap();
        assert_eq!(doc.chunks[5].offset_begin, 35);
        assert_eq!(doc.chunks[5].offset_end, 40);

        doc.load_chunk(30, 43).unwrap();
        assert_eq!(doc.chunks[5].offset_begin, 35);
        assert_eq!(doc.chunks[5].offset_end, 40);

        doc.load_chunk(30, 45).unwrap();
        assert_eq!(doc.chunks[5].offset_begin, 35);
        assert_eq!(doc.chunks[5].offset_end, 40);

        doc.load_chunk(6, 31).unwrap();
        assert_eq!(doc.chunks.len(), 6);
    }

    #[test]
    fn test_load_chunk_drain() {
        let cursor = Cursor::new("1234\n1234\n1234\n1234\n1234\n1234\n1234\n1234\n");
        let mut doc = Document::new(cursor.clone()).unwrap();
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.last_line.as_ref().unwrap(), "1234\n");
        doc.chunks.pop();

        doc.load_chunk(0, 11).unwrap();
        assert_eq!(doc.chunks[0].offset_begin, 0);
        assert_eq!(doc.chunks[0].offset_end, 10);

        doc.load_chunk(15, 21).unwrap();
        assert_eq!(doc.chunks[1].offset_begin, 15);
        assert_eq!(doc.chunks[1].offset_end, 20);

        doc.load_chunk(23, 32).unwrap();
        assert_eq!(doc.chunks.len(), 3);
        assert_eq!(doc.chunks[2].offset_begin, 25);
        assert_eq!(doc.chunks[2].offset_end, 30);

        doc.load_chunk(35, 40).unwrap();
        assert_eq!(doc.chunks.len(), 3);

        doc.load_chunk(12, 32).unwrap();
        assert_eq!(doc.chunks.len(), 2);
        assert_eq!(doc.chunks[1].offset_begin, 15);
        assert_eq!(doc.chunks[1].offset_end, 30);
    }
}
