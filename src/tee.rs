use std::io;
use std::io::{Read, Write};
use std::fs::{File, rename};
use std::path::PathBuf;
use iron::response::{WriteBody, ResponseBody};

pub struct Tee<R: Read + Send> {
    reader: R,
    file: File,
    temp_filename: PathBuf,
    final_filename: PathBuf,
}

impl<R: Read + Send> Tee<R> {
    pub fn new(reader: R, final_filename: PathBuf) -> Self {
        let temp_filename = final_filename.with_file_name(
            format!(".tmp.{}", final_filename.file_name().unwrap().to_str().unwrap()));
        assert!(!temp_filename.exists(), "file exists: {:?}", temp_filename);
        let file = File::create(&temp_filename).unwrap();
        Tee {
            reader: reader,
            file: file,
            temp_filename: temp_filename,
            final_filename: final_filename,
        }
    }
}

impl<R: Read + Send> WriteBody for Tee<R> {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        let mut buf = [0; 8 * 1024];
        loop {
            let len = match self.reader.read(&mut buf) {
                Ok(0) => {
                    assert!(!self.final_filename.exists(), "file exists: {:?}", self.final_filename);
                    rename(&self.temp_filename, &self.final_filename)?;
                    return Ok(());
                },
                Ok(len) => len,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            self.file.write_all(&buf[..len])?;
            res.write_all(&buf[..len])?;
        }
    }
}
