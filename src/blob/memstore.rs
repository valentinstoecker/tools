use std::{
    collections::HashMap,
    io::{Read, Result, Write},
    rc::Rc,
};

use sha1::{Digest, Sha1};

use super::{BlobStore, Hash};

pub struct MemStore(HashMap<Hash, Rc<[u8]>>);

impl MemStore {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl BlobStore for MemStore {
    fn get<W: Write>(&self, h: &Hash, w: &mut W) -> Result<()> {
        match self.0.get(h) {
            Some(buf) => w.write_all(buf),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )),
        }
    }
    fn put<R: Read>(&mut self, r: &mut R) -> Result<Hash> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        let hash = Hash(Sha1::digest(&buf).into());
        self.0.insert(hash.clone(), buf.into());
        Ok(hash)
    }
    fn get_buf(&self, h: &Hash) -> Result<Rc<[u8]>> {
        match self.0.get(h) {
            Some(buf) => Ok(buf.clone()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )),
        }
    }
    fn get_vec(&self, h: &Hash) -> Result<Vec<u8>> {
        match self.0.get(h) {
            Some(buf) => Ok(buf.to_vec()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )),
        }
    }
    fn put_buf(&mut self, buf: &[u8]) -> Result<Hash> {
        let hash = Hash(Sha1::digest(buf).into());
        self.0.insert(hash.clone(), buf.into());
        Ok(hash)
    }
    fn put_vec(&mut self, buf: Vec<u8>) -> Result<Hash> {
        let hash = Hash(Sha1::digest(&buf).into());
        self.0.insert(hash.clone(), buf.into());
        Ok(hash)
    }
}
