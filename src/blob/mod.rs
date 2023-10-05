use std::{
    fmt::Display,
    io::{Read, Result, Write},
    rc::Rc,
};

pub mod filestore;
pub mod memstore;
pub mod zipped;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Hash([u8; 20]);

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for b in self.0 {
            s.push_str(&format!("{:02x}", b));
        }
        write!(f, "{}", s)
    }
}

pub trait BlobStore {
    fn put<R: Read>(&mut self, r: &mut R) -> Result<Hash>;
    fn get<W: Write>(&self, h: &Hash, w: &mut W) -> Result<()>;

    fn put_str(&mut self, s: &str) -> Result<Hash> {
        self.put(&mut s.as_bytes())
    }

    fn put_buf(&mut self, buf: &[u8]) -> Result<Hash> {
        self.put(&mut buf.as_ref())
    }

    fn put_vec(&mut self, buf: Vec<u8>) -> Result<Hash> {
        self.put(&mut buf.as_slice())
    }

    fn get_buf(&self, h: &Hash) -> Result<Rc<[u8]>> {
        Ok(self.get_vec(h)?.into())
    }

    fn get_str(&self, h: &Hash) -> Result<String> {
        match String::from_utf8(self.get_vec(h)?) {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    fn get_vec(&self, h: &Hash) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.get(h, &mut buf)?;
        Ok(buf)
    }
}
