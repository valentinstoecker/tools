use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Result, Write},
    path::PathBuf,
};

use sha1::{Digest, Sha1};
use tempfile::NamedTempFile;

use crate::utils::ioutil::TeeWriter;

use super::{BlobStore, Hash};

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
    fn get<W: Write>(&self, h: &Hash, w: &mut W) -> Result<()> {
        let str = h.to_string();
        let path = self.path.join(&str[0..2]).join(&str[2..]);
        let mut r = BufReader::new(File::open(path)?);
        io::copy(&mut r, w)?;
        Ok(())
    }
    fn put<R: Read>(&mut self, r: &mut R) -> Result<Hash> {
        let mut sha = Sha1::new();
        let mut temp_file = NamedTempFile::new_in(&self.path)?;
        {
            let buf_file = BufWriter::new(&mut temp_file);
            let buf_sha = BufWriter::new(&mut sha);
            let mut tee_w = TeeWriter::new(buf_file, buf_sha);
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
