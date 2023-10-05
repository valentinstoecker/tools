use std::io::{Read, Result, Write};

use flate2::{read::ZlibEncoder, write::ZlibDecoder};

use super::BlobStore;

pub struct ZippedStore<B: BlobStore> {
    inner: B,
}

impl<B: BlobStore> ZippedStore<B> {
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B: BlobStore> BlobStore for ZippedStore<B> {
    fn get<W: Write>(&self, h: &super::Hash, w: &mut W) -> Result<()> {
        let mut dec = ZlibDecoder::new(w);
        self.inner.get(h, &mut dec)
    }
    fn put<R: Read>(&mut self, r: &mut R) -> Result<super::Hash> {
        let mut enc = ZlibEncoder::new(r, flate2::Compression::default());
        self.inner.put(&mut enc)
    }
}
