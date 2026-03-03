use std::io::{self, Write, Read};
use zstd::stream::write::Encoder;
use byteorder::{LittleEndian, WriteBytesExt};

pub const BLB1_MAGIC: &[u8; 4] = b"BLB1";

pub struct Blb1Writer<W: Write> {
    encoder: Encoder<'static, W>,
}

impl<W: Write> Blb1Writer<W> {
    pub fn new(writer: W, level: i32) -> io::Result<Self> {
        let mut encoder = Encoder::new(writer, level)?;
        encoder.write_all(BLB1_MAGIC)?;
        Ok(Self { encoder })
    }

    pub fn write_record(&mut self, data: &[u8]) -> io::Result<()> {
        // Record format: DataLen(u64) | Data
        self.encoder.write_u64::<LittleEndian>(data.len() as u64)?;
        self.encoder.write_all(data)?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()
    }

    pub fn finish(self) -> io::Result<W> {
        self.encoder.finish()
    }
}

pub struct Blb1Reader<R: io::Read> {
    decoder: zstd::stream::read::Decoder<'static, io::BufReader<R>>,
}

impl<R: io::Read> Blb1Reader<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        let mut decoder = zstd::stream::read::Decoder::new(reader)?;
        let mut magic = [0u8; 4];
        decoder.read_exact(&mut magic)?;
        if &magic != BLB1_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid BLB1 magic"));
        }
        Ok(Self { decoder })
    }

    pub fn next_record(&mut self) -> io::Result<Option<Vec<u8>>> {
        use byteorder::ReadBytesExt;
        let len = match self.decoder.read_u64::<LittleEndian>() {
            Ok(l) => l,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };

        let mut buf = vec![0u8; len as usize];
        self.decoder.read_exact(&mut buf)?;
        Ok(Some(buf))
    }
}
