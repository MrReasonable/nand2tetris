use std::io::{BufReader, BufRead, Error, Seek};
use std::fs::File;

pub trait Read: Iterator {
    fn reset(&mut self);
    fn eof(&mut self) -> bool;
}

pub struct FileReader {
    buf_reader: BufReader<File>,
    buffer: String,
    eof: bool
}

impl FileReader {
    pub fn new(path: &str) -> Result<FileReader, Error> {
        let fh = File::open(&path);
        let fh = match fh {
            Ok(file) => file,
            Err(e) => return Err(
                Error::new(e.kind(), format!("Unable to open {}. {}", &path, e))
            ),
        };
        
        let mut buf_reader = BufReader::new(fh);
        let mut buffer = String::new();
        let eof = match buf_reader.read_line(&mut buffer) {
            Ok(0) => true,
            Ok(_) => false,
            Err(e) => return Err(e)
        };

        let buffer = String::from(buffer.trim());
        
        Ok(FileReader {
            buf_reader,
            buffer,
            eof
        })
    }
}

impl Read for FileReader {
    fn eof(&mut self) -> bool {
        self.eof
    }

    fn reset(&mut self) {
        self.buf_reader.rewind().unwrap();
        self.eof = false;
        self.next();
    }
}

impl Iterator for FileReader {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            None
        } else {
            let buff = String::from(&self.buffer);
            self.buffer = String::new();
            self.eof = matches!(self.buf_reader.read_line(&mut self.buffer).unwrap(), 0);
            self.buffer = String::from(self.buffer.trim_end());
            Some(buff)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn it_reports_file_not_found() {
        match FileReader::new("nopath") {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(ErrorKind::NotFound, e.kind())
        };
    }

    #[test]
    fn it_loads_file() {
        let fr = FileReader::new("test_files/empty_file.test");
        match fr {
            Err(e) => panic!("{}", e),
            _ => assert_eq!(true, true)
        }
    }

    #[test]
    fn it_reports_eof_on_empty_file() {
        let mut fr = FileReader::new("test_files/empty_file.test").unwrap();
        assert!(fr.eof());
    }

    #[test]
    fn it_reports_not_eof_on_initial_load_of_one_line_file() {
        let mut fr = FileReader::new("test_files/one_line_file.test").unwrap();
        assert!(!fr.eof());
    }

    #[test]
    fn it_gets_next_line_from_file() {
        let mut fr = FileReader::new("test_files/one_line_file.test").unwrap();
        let line = fr.next().unwrap();
        assert_eq!(line, "This is a single line file.");

        let mut fr = FileReader::new("test_files/two_line_file.test").unwrap();
        let line1 = fr.next().unwrap();
        assert_eq!(line1, "This is line1.");
        let line2 = fr.next().unwrap();
        assert_eq!(line2, "This is line2.");
    }

    #[test]
    fn it_reports_eof_after_taking_one_line_from_single_line_file() {
        let mut fr = FileReader::new("test_files/one_line_file.test").unwrap();
        fr.next();
        assert!(fr.eof());
    }

    #[test]
    fn it_reports_eof_after_taking_two_lines_from_two_line_file() {
        let mut fr = FileReader::new("test_files/two_line_file.test").unwrap();
        fr.next();
        assert!(!fr.eof());
        fr.next();
        assert!(fr.eof());
    }

    #[test]
    fn it_reports_not_eof_after_reset() {
        let mut fr = FileReader::new("test_files/two_line_file.test").unwrap();
        fr.reset();
        assert!(!fr.eof());
        fr.next();
        fr.next();
        fr.reset();
        assert!(!fr.eof());
    }

    #[test]
    fn it_reads_first_line_after_reset() {
        let mut fr = FileReader::new("test_files/two_line_file.test").unwrap();
        fr.next();
        fr.next();
        fr.reset();
        let line1 = fr.next().unwrap();
        assert_eq!(line1, "This is line1.");
    }
}
