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

impl Hash {
    pub fn new(buf: [u8; 20]) -> Self {
        Self(buf)
    }
    pub fn buf(&self) -> &[u8] {
        &self.0
    }
}

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

#[cfg(test)]
mod tests {
    use super::{filestore::FileStore, memstore::MemStore, zipped::ZippedStore, BlobStore};
    use std::{fs, io::Result};

    fn test_blob_store<T: BlobStore>(mut bs: T) -> Result<()> {
        let h_1 = bs.put_str("hello world")?;
        let s_1 = bs.get_str(&h_1)?;
        assert_eq!(s_1, "hello world");
        let h_2 = bs.put_str("hello world")?;
        let s_2 = bs.get_str(&h_2)?;
        assert_eq!(h_1, h_2);
        assert_eq!(s_1, s_2);
        let h_3 = bs.put_str("hello world!")?;
        let s_3 = bs.get_str(&h_3)?;
        assert_ne!(h_1, h_3);
        assert_ne!(s_1, s_3);
        let mut hashes = Vec::with_capacity(1000);
        for i in 0..1000 {
            let h = bs.put_str(&format!("hello world {}", i).repeat(1000))?;
            hashes.push(h);
        }
        for i in 0..1000 {
            let s = bs.get_str(&hashes[i])?;
            assert_eq!(s, format!("hello world {}", i).repeat(1000));
        }
        Ok(())
    }

    #[test]
    fn test_file_store() -> Result<()> {
        let bs = FileStore::new("test_store".into())?;
        test_blob_store(bs)?;
        fs::remove_dir_all("test_store")
    }

    #[test]
    fn test_mem_store() -> Result<()> {
        let bs = MemStore::new();
        test_blob_store(bs)
    }

    #[test]
    fn test_zipped_store() -> Result<()> {
        let bs = ZippedStore::new(MemStore::new());
        test_blob_store(bs)
    }
}
