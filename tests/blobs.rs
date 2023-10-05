use std::{fs, io::Result};
use tools::blob::{filestore::FileStore, memstore::MemStore, zipped::ZippedStore, BlobStore};

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
        let h = bs.put_str(&format!("hello world {}", i))?;
        hashes.push(h);
    }
    for i in 0..1000 {
        let s = bs.get_str(&hashes[i])?;
        assert_eq!(s, format!("hello world {}", i));
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
