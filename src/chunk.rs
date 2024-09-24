#[derive(Debug)]
pub struct Chunk {
    pub offset_begin: usize,
    pub offset_end: usize,
    pub rows: Vec<String>,
}

impl Chunk {
    pub fn build_chunk(
        content: &str,
        content_offset: usize,
        drop_first: bool,
        drop_last: bool,
    ) -> Chunk {
        let mut cur_index = 0;
        if drop_first {
            let first_line_break = content.find('\n');
            cur_index = first_line_break.unwrap() + 1;
        }
        let offset_begin = content_offset + cur_index;
        let mut rows = vec![];
        while let Some(pos) = content[cur_index..].find('\n') {
            let next_line_break = cur_index + pos;
            rows.push(content[cur_index..next_line_break].to_string());
            cur_index = next_line_break + 1;
        }
        if !drop_last && cur_index < content.len() {
            rows.push(content[cur_index..].to_string());
            cur_index += content[cur_index..].len();
        }
        let offset_end = content_offset + cur_index;
        Chunk {
            offset_begin,
            offset_end,
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn test_build_chunk() {
        let content = "123456\n12345\n12\n\n123456";

        let chunk = Chunk::build_chunk(content, 2, false, false);
        assert_eq!(chunk.offset_begin, 2);
        assert_eq!(chunk.offset_end, 25);
        assert_eq!(chunk.rows, vec!["123456", "12345", "12", "", "123456"]);

        let chunk = Chunk::build_chunk(content, 2, true, false);
        assert_eq!(chunk.offset_begin, 9);
        assert_eq!(chunk.offset_end, 25);
        assert_eq!(chunk.rows, vec!["12345", "12", "", "123456"]);

        let chunk = Chunk::build_chunk(content, 2, false, true);
        assert_eq!(chunk.offset_begin, 2);
        assert_eq!(chunk.offset_end, 19);
        assert_eq!(chunk.rows, vec!["123456", "12345", "12", ""]);

        let chunk = Chunk::build_chunk(content, 2, true, true);
        assert_eq!(chunk.offset_begin, 9);
        assert_eq!(chunk.offset_end, 19);
        assert_eq!(chunk.rows, vec!["12345", "12", ""]);

        let content = "\nabc\n12\n\n\n12345\n";

        let chunk = Chunk::build_chunk(content, 1, false, false);
        assert_eq!(chunk.offset_begin, 1);
        assert_eq!(chunk.offset_end, 17);
        assert_eq!(chunk.rows, vec!["", "abc", "12", "", "", "12345"]);

        let chunk = Chunk::build_chunk(content, 1, true, false);
        assert_eq!(chunk.offset_begin, 2);
        assert_eq!(chunk.offset_end, 17);
        assert_eq!(chunk.rows, vec!["abc", "12", "", "", "12345"]);

        let chunk = Chunk::build_chunk(content, 1, false, true);
        assert_eq!(chunk.offset_begin, 1);
        assert_eq!(chunk.offset_end, 17);
        assert_eq!(chunk.rows, vec!["", "abc", "12", "", "", "12345"]);

        let chunk = Chunk::build_chunk(content, 1, true, true);
        assert_eq!(chunk.offset_begin, 2);
        assert_eq!(chunk.offset_end, 17);
        assert_eq!(chunk.rows, vec!["abc", "12", "", "", "12345"]);
    }
}
