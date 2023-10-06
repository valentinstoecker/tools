use std::io::{Read, Result, Write};

use flate2::{read::ZlibEncoder, write::ZlibDecoder, Compression};

use super::{BlobStore, Hash};

pub struct ZippedStore<B: BlobStore>(B);

impl<B: BlobStore> ZippedStore<B> {
    pub fn new(inner: B) -> Self {
        Self(inner)
    }
}

impl<B: BlobStore> BlobStore for ZippedStore<B> {
    fn get<W: Write>(&self, h: &Hash, w: &mut W) -> Result<()> {
        let mut dec = ZlibDecoder::new(w);
        self.0.get(h, &mut dec)
    }
    fn put<R: Read>(&mut self, r: &mut R) -> Result<Hash> {
        let mut enc = ZlibEncoder::new(r, Compression::default());
        self.0.put(&mut enc)
    }
}
