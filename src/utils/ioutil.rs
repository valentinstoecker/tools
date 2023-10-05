use std::io::{self, Write};

pub struct TeeWriter<W: Write, V: Write> {
    w1: W,
    w2: V,
}

impl<W: Write, V: Write> TeeWriter<W, V> {
    pub fn new(w1: W, w2: V) -> Self {
        Self { w1, w2 }
    }
}

impl<W: Write, V: Write> Write for TeeWriter<W, V> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n1 = self.w1.write(buf)?;
        let n2 = self.w2.write(buf)?;
        if n1 < n2 {
            self.w1.write_all(&buf[n1..n2])?;
        } else if n1 > n2 {
            self.w2.write_all(&buf[n2..n1])?;
        }
        Ok(n1.max(n2))
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w1.flush()?;
        self.w2.flush()?;
        Ok(())
    }
}

pub struct CapWriter<W: Write> {
    cap: usize,
    w: W,
}

impl<W: Write> CapWriter<W> {
    pub fn new(cap: usize, w: W) -> Self {
        Self { cap, w }
    }
}

impl<W: Write> Write for CapWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.cap {
            self.w.write(&buf[..self.cap])
        } else {
            self.w.write(buf)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

pub struct LogWriter<W: Write> {
    s: String,
    w: W,
}

impl<W: Write> LogWriter<W> {
    pub fn new(s: String, w: W) -> Self {
        Self { s, w }
    }
}

impl<W: Write> Write for LogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        println!("{}.write([u8;{}])", self.s, buf.len());
        self.w.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

#[test]
fn test_tee_writer() -> io::Result<()> {
    use std::io::BufWriter;

    type TestBuf<W> = CapWriter<LogWriter<BufWriter<LogWriter<W>>>>;

    fn test_buf_w<W: Write>(c: usize, s: String, w: W) -> TestBuf<W> {
        CapWriter::new(
            c,
            LogWriter::new(
                s.clone(),
                BufWriter::new(LogWriter::new(s.to_uppercase(), w)),
            ),
        )
    }

    let test_buf = b"1234567890\n".repeat(10000);
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    {
        let buf1_w = test_buf_w(4096, "buf1".to_string(), &mut buf1);
        let buf2_w = test_buf_w(2048, "buf2".to_string(), &mut buf2);
        let tee = TeeWriter::new(buf1_w, buf2_w);
        let mut tee_lw = CapWriter::new(8192, LogWriter::new("tee_".to_string(), tee));
        tee_lw.write_all(&test_buf)?;
        tee_lw.flush()?;
    }
    assert_eq!(buf1, test_buf);
    assert_eq!(buf1, buf2);
    Ok(())
}
