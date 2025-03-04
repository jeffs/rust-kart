use std::io;

struct TrimmedLine {
    is_comment: bool,
    is_empty_comment: bool,
}

fn trim(line: &str) -> TrimmedLine {
    let line = line.trim();
    let is_comment = line.starts_with("//");
    let is_empty_comment = is_comment && ["//", "///", "//!"].contains(&line);
    TrimmedLine {
        is_comment,
        is_empty_comment,
    }
}

pub struct TrailingEmptyCommentLineNos<R: io::BufRead> {
    is_done: bool,
    line_no: usize,
    line: String,
    reader: R,
}

impl<R: io::BufRead> TrailingEmptyCommentLineNos<R> {
    pub fn from_buf(reader: R) -> TrailingEmptyCommentLineNos<R> {
        TrailingEmptyCommentLineNos {
            is_done: false,
            line_no: 0,
            line: String::new(),
            reader,
        }
    }
}

impl<R: io::BufRead> Iterator for TrailingEmptyCommentLineNos<R> {
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        loop {
            let last_no = self.line_no;
            let last = trim(&self.line);

            self.line.clear();
            match self.reader.read_line(&mut self.line) {
                Ok(0) => return last.is_empty_comment.then_some(Ok(last_no)),
                Ok(_) => {
                    self.line_no += 1;
                    let line = trim(self.line.trim());
                    if last.is_empty_comment && (!line.is_comment || line.is_empty_comment) {
                        return Some(Ok(last_no));
                    }
                }
                err @ Err(_) => return Some(err),
            }
        }
    }
}
