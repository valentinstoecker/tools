use std::{
    fs::{self, File},
    io::{self, Read, Result, Write},
    path::PathBuf,
};

use sha1::{Digest, Sha1};
use tempfile::NamedTempFile;

use super::{BlobStore, Hash};

struct TeeWriter<'a, W: Write, V: Write> {
    w1: &'a mut W,
    w2: &'a mut V,
}

impl<'a, W: Write, V: Write> Write for TeeWriter<'a, W, V> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w1.write_all(buf)?;
        self.w2.write_all(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w1.flush()?;
        self.w2.flush()?;
        Ok(())
    }
}

pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(path.clone())?;
        Ok(Self { path })
    }
}

impl BlobStore for FileStore {
    fn get<W: Write>(&self, h: &super::Hash, w: &mut W) -> Result<()> {
        let str = h.to_string();
        let path = self.path.join(&str[0..2]).join(&str[2..]);
        let mut f = File::open(path)?;
        io::copy(&mut f, w)?;
        Ok(())
    }
    fn put<R: Read>(&mut self, r: &mut R) -> Result<super::Hash> {
        let mut sha = Sha1::new();
        let mut temp_file = NamedTempFile::new_in(&self.path)?;
        {
            let mut tee_w = TeeWriter {
                w1: &mut io::BufWriter::new(&mut sha),
                w2: &mut io::BufWriter::new(&mut temp_file),
            };
            io::copy(r, &mut tee_w)?;
        }
        let hash = Hash(sha.finalize().into());
        let str = hash.to_string();
        fs::create_dir_all(self.path.join(&str[0..2]))?;
        let path = self.path.join(&str[0..2]).join(&str[2..]);
        temp_file.persist(path)?;
        Ok(hash)
    }
}
