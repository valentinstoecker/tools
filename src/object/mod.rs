use std::io::{Read, Result, Write};
use std::marker::PhantomData;

use crate::blob::{BlobStore, Hash};

pub trait SerDe
where
    Self: Sized,
{
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()>;
    fn deserialize<R: Read>(r: &mut R) -> Result<Self>;
}

impl SerDe for Hash {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(self.buf())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 20];
        r.read_exact(&mut buf)?;
        Ok(Hash::new(buf))
    }
}

impl SerDe for String {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.len().serialize(w)?;
        w.write_all(self.as_bytes())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::deserialize(r)?;
        let mut buf = vec![0; len];
        r.read_exact(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

impl SerDe for u8 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&[*self])
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 1];
        r.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl SerDe for u16 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.to_be_bytes())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 2];
        r.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}

impl SerDe for u32 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.to_be_bytes())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl SerDe for u64 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.to_be_bytes())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }
}

impl SerDe for usize {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        (*self as u64).serialize(w)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok(u64::deserialize(r)? as usize)
    }
}

pub struct ObjRef<T> {
    hash: Hash,
    t: PhantomData<T>,
}

impl<T> SerDe for ObjRef<T> {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.hash.serialize(w)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let hash = Hash::deserialize(r)?;
        Ok(Self {
            hash,
            t: PhantomData,
        })
    }
}

impl<T: SerDe> SerDe for Vec<T> {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.len().serialize(w)?;
        for t in self {
            t.serialize(w)?;
        }
        Ok(())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::deserialize(r)?;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(T::deserialize(r)?);
        }
        Ok(v)
    }
}

impl<T: SerDe, U: SerDe> SerDe for (T, U) {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.0.serialize(w)?;
        self.1.serialize(w)?;
        Ok(())
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok((T::deserialize(r)?, U::deserialize(r)?))
    }
}

pub trait Obj
where
    Self: Sized,
{
    fn store<S: BlobStore>(&self, store: &mut S) -> Result<ObjRef<Self>>;
    fn load<S: BlobStore>(store: &S, r: &ObjRef<Self>) -> Result<Self>;
}

impl<O: Obj> Obj for Vec<O> {
    fn store<S: BlobStore>(&self, store: &mut S) -> Result<ObjRef<Self>> {
        let mut hash_list = Vec::with_capacity(self.len());
        for o in self {
            hash_list.push(o.store(store)?);
        }
        let mut buf = Vec::new();
        hash_list.serialize(&mut buf)?;
        Ok(ObjRef {
            hash: store.put_vec(buf)?,
            t: PhantomData,
        })
    }
    fn load<S: BlobStore>(store: &S, r: &ObjRef<Self>) -> Result<Self> {
        let buf = store.get_vec(&r.hash)?;
        let hash_list = Vec::<ObjRef<O>>::deserialize(&mut buf.as_slice())?;
        let mut v = Vec::with_capacity(hash_list.len());
        for r in hash_list {
            v.push(O::load(store, &r)?);
        }
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{Read, Result, Write},
        marker::PhantomData,
    };

    use crate::blob::{filestore::FileStore, zipped::ZippedStore, BlobStore};

    use super::{Obj, ObjRef, SerDe};

    #[derive(PartialEq, Debug)]
    enum TreeEntry {
        Blob(Buf),
        Tree(Tree),
    }

    enum TreeEntrySer {
        Blob(ObjRef<Buf>),
        Tree(ObjRef<Tree>),
    }

    impl SerDe for TreeEntrySer {
        fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
            match self {
                TreeEntrySer::Blob(r) => {
                    w.write_all(&[0])?;
                    r.serialize(w)?;
                }
                TreeEntrySer::Tree(r) => {
                    w.write_all(&[1])?;
                    r.serialize(w)?;
                }
            }
            Ok(())
        }
        fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
            let mut buf = [0; 1];
            r.read_exact(&mut buf)?;
            match buf[0] {
                0 => Ok(TreeEntrySer::Blob(ObjRef::deserialize(r)?)),
                1 => Ok(TreeEntrySer::Tree(ObjRef::deserialize(r)?)),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "invalid tree entry",
                )),
            }
        }
    }

    #[derive(PartialEq, Debug)]
    struct Buf(Vec<u8>);

    impl Obj for Buf {
        fn store<S: BlobStore>(&self, store: &mut S) -> Result<ObjRef<Self>> {
            Ok(ObjRef {
                hash: store.put_vec(self.0.clone())?,
                t: PhantomData,
            })
        }
        fn load<S: BlobStore>(store: &S, r: &ObjRef<Self>) -> Result<Self> {
            Ok(Buf(store.get_vec(&r.hash)?))
        }
    }

    #[derive(PartialEq, Debug)]
    struct Tree {
        entries: HashMap<String, TreeEntry>,
    }

    struct TreeSer {
        entries: Vec<(String, TreeEntrySer)>,
    }

    impl SerDe for TreeSer {
        fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
            self.entries.serialize(w)
        }
        fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
            Ok(Self {
                entries: Vec::<(String, TreeEntrySer)>::deserialize(r)?,
            })
        }
    }

    impl Obj for Tree {
        fn store<S: BlobStore>(&self, store: &mut S) -> Result<ObjRef<Self>> {
            let mut to_ser = Vec::new();
            for (name, entry) in &self.entries {
                match entry {
                    TreeEntry::Blob(buf) => {
                        to_ser.push((name.clone(), TreeEntrySer::Blob(buf.store(store)?)));
                    }
                    TreeEntry::Tree(tree) => {
                        to_ser.push((name.clone(), TreeEntrySer::Tree(tree.store(store)?)));
                    }
                }
            }
            to_ser.sort_by(|(name1, _), (name2, _)| name1.cmp(name2));
            let mut buf = Vec::new();
            TreeSer { entries: to_ser }.serialize(&mut buf)?;
            Ok(ObjRef {
                hash: store.put_vec(buf)?,
                t: PhantomData,
            })
        }
        fn load<S: BlobStore>(store: &S, r: &ObjRef<Self>) -> Result<Self> {
            let buf = store.get_vec(&r.hash)?;
            let ser = TreeSer::deserialize(&mut buf.as_slice())?;
            let mut entries = HashMap::new();
            for (name, entry_ser) in ser.entries {
                match entry_ser {
                    TreeEntrySer::Blob(r) => {
                        entries.insert(name, TreeEntry::Blob(Buf::load(store, &r)?));
                    }
                    TreeEntrySer::Tree(r) => {
                        entries.insert(name, TreeEntry::Tree(Tree::load(store, &r)?));
                    }
                }
            }
            Ok(Self { entries })
        }
    }

    #[test]
    fn test_obj() -> Result<()> {
        fn fib_tree(n: usize) -> Tree {
            let mut entries = HashMap::new();
            if n == 0 {
                entries.insert("fib 0".into(), TreeEntry::Blob(Buf(vec![0])));
            } else if n == 1 {
                entries.insert("fib 1".into(), TreeEntry::Blob(Buf(vec![1])));
            } else {
                entries.insert(format!("fib {}", n - 1), TreeEntry::Tree(fib_tree(n - 1)));
                entries.insert(format!("fib {}", n - 2), TreeEntry::Tree(fib_tree(n - 2)));
            }
            Tree { entries }
        }
        let mut store = ZippedStore::new(FileStore::new("test_store".into())?);
        let test_tree = fib_tree(10);
        let r = test_tree.store(&mut store)?;
        let test_tree_2 = Tree::load(&store, &r)?;
        assert_eq!(test_tree.entries, test_tree_2.entries);
        println!("{:?}", test_tree_2);
        Ok(())
    }
}
