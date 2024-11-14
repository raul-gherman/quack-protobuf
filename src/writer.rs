use crate::errors::{Error, Result};
use crate::message::MessageWrite;
use byteorder_lite::{ByteOrder, LittleEndian as LE};

#[cfg(feature = "std")]
use byteorder_lite::WriteBytesExt;

pub struct Writer<W: WriterBackend> {
    inner: W,
}

impl<W: WriterBackend> Writer<W> {
    /// Creates a new `ProtobufWriter`
    pub fn new(w: W) -> Writer<W> {
        Writer { inner: w }
    }

    /// Writes a byte which is NOT internally coded as a `varint`
    pub fn write_u8(&mut self, byte: u8) -> Result<()> {
        self.inner.pb_write_u8(byte)
    }

    /// Writes a `varint` (compacted `u64`)
    pub fn write_varint(&mut self, mut v: u64) -> Result<()> {
        while v > 0x7f {
            self.inner.pb_write_u8(((v as u8) & 0x7f) | 0x80)?;
            v >>= 7;
        }
        self.inner.pb_write_u8(v as u8)
    }

    /// Writes a tag, which represents both the field number and the wire type
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_tag(&mut self, tag: u32) -> Result<()> {
        self.write_varint(tag as u64)
    }

    /// Writes a `int32` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_int32(&mut self, v: i32) -> Result<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `int64` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_int64(&mut self, v: i64) -> Result<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `uint32` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_uint32(&mut self, v: u32) -> Result<()> {
        self.write_varint(v as u64)
    }

    /// Writes a `uint64` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_uint64(&mut self, v: u64) -> Result<()> {
        self.write_varint(v)
    }

    /// Writes a `sint32` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_sint32(&mut self, v: i32) -> Result<()> {
        self.write_varint(((v << 1) ^ (v >> 31)) as u64)
    }

    /// Writes a `sint64` which is internally coded as a `varint`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_sint64(&mut self, v: i64) -> Result<()> {
        self.write_varint(((v << 1) ^ (v >> 63)) as u64)
    }

    /// Writes a `fixed64` which is little endian coded `u64`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_fixed64(&mut self, v: u64) -> Result<()> {
        self.inner.pb_write_u64(v)
    }

    /// Writes a `fixed32` which is little endian coded `u32`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_fixed32(&mut self, v: u32) -> Result<()> {
        self.inner.pb_write_u32(v)
    }

    /// Writes a `sfixed64` which is little endian coded `i64`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_sfixed64(&mut self, v: i64) -> Result<()> {
        self.inner.pb_write_i64(v)
    }

    /// Writes a `sfixed32` which is little endian coded `i32`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_sfixed32(&mut self, v: i32) -> Result<()> {
        self.inner.pb_write_i32(v)
    }

    /// Writes a `float`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_float(&mut self, v: f32) -> Result<()> {
        self.inner.pb_write_f32(v)
    }

    /// Writes a `double`
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_double(&mut self, v: f64) -> Result<()> {
        self.inner.pb_write_f64(v)
    }

    /// Writes a `bool` 1 = true, 0 = false
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_bool(&mut self, v: bool) -> Result<()> {
        self.inner.pb_write_u8(u8::from(v))
    }

    /// Writes an `enum` converting it to a `i32` first
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_enum(&mut self, v: i32) -> Result<()> {
        self.write_int32(v)
    }

    /// Writes `bytes`: length first then the chunk of data
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_varint(bytes.len() as u64)?;
        self.inner.pb_write_all(bytes)
    }

    /// Writes `string`: length first then the chunk of data
    #[cfg_attr(feature = "std", inline(always))]
    pub fn write_string(&mut self, s: &str) -> Result<()> {
        self.write_bytes(s.as_bytes())
    }

    /// Writes a message which implements `MessageWrite` without adding the length prefix
    #[cfg_attr(feature = "std", inline)]
    pub fn write_message<M: MessageWrite>(&mut self, m: &M) -> Result<()> {
        m.write_message(self)
    }

    /// Writes another item prefixed with tag
    #[cfg_attr(feature = "std", inline)]
    pub fn write_with_tag<F>(&mut self, tag: u32, mut write: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        self.write_tag(tag)?;
        write(self)
    }

    /// Write entire map
    pub fn write_map<FK, FV>(
        &mut self,
        size: usize,
        tag_key: u32,
        mut write_key: FK,
        tag_val: u32,
        mut write_val: FV,
    ) -> Result<()>
    where
        FK: FnMut(&mut Self) -> Result<()>,
        FV: FnMut(&mut Self) -> Result<()>,
    {
        self.write_varint(size as u64)?;
        self.write_tag(tag_key)?;
        write_key(self)?;
        self.write_tag(tag_val)?;
        write_val(self)
    }
}

/// Writer backend abstraction
pub trait WriterBackend {
    /// Write a u8
    fn pb_write_u8(&mut self, x: u8) -> Result<()>;

    /// Write a u32
    fn pb_write_u32(&mut self, x: u32) -> Result<()>;

    /// Write a i32
    fn pb_write_i32(&mut self, x: i32) -> Result<()>;

    /// Write a f32
    fn pb_write_f32(&mut self, x: f32) -> Result<()>;

    /// Write a u64
    fn pb_write_u64(&mut self, x: u64) -> Result<()>;

    /// Write a i64
    fn pb_write_i64(&mut self, x: i64) -> Result<()>;

    /// Write a f64
    fn pb_write_f64(&mut self, x: f64) -> Result<()>;

    /// Write all bytes in buf
    fn pb_write_all(&mut self, buf: &[u8]) -> Result<()>;
}

/// A writer backend for byte buffers
pub struct BytesWriter<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> BytesWriter<'a> {
    /// Create a new BytesWriter to write into `buf`
    pub fn new(buf: &'a mut [u8]) -> BytesWriter<'a> {
        BytesWriter { buf, cursor: 0 }
    }
}

impl<'a> WriterBackend for BytesWriter<'a> {
    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_u8(&mut self, x: u8) -> Result<()> {
        if self.buf.len() - self.cursor < 1 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            self.buf[self.cursor] = x;
            self.cursor += 1;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_u32(&mut self, x: u32) -> Result<()> {
        if self.buf.len() - self.cursor < 4 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_u32(&mut self.buf[self.cursor..], x);
            self.cursor += 4;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_i32(&mut self, x: i32) -> Result<()> {
        if self.buf.len() - self.cursor < 4 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_i32(&mut self.buf[self.cursor..], x);
            self.cursor += 4;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_f32(&mut self, x: f32) -> Result<()> {
        if self.buf.len() - self.cursor < 4 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_f32(&mut self.buf[self.cursor..], x);
            self.cursor += 4;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_u64(&mut self, x: u64) -> Result<()> {
        if self.buf.len() - self.cursor < 8 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_u64(&mut self.buf[self.cursor..], x);
            self.cursor += 8;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_i64(&mut self, x: i64) -> Result<()> {
        if self.buf.len() - self.cursor < 8 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_i64(&mut self.buf[self.cursor..], x);
            self.cursor += 8;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_f64(&mut self, x: f64) -> Result<()> {
        if self.buf.len() - self.cursor < 8 {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            LE::write_f64(&mut self.buf[self.cursor..], x);
            self.cursor += 8;
            Ok(())
        }
    }

    #[cfg_attr(feature = "std", inline(always))]
    fn pb_write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.buf.len() - self.cursor < buf.len() {
            Err(Error::UnexpectedEndOfBuffer)
        } else {
            self.buf[self.cursor..self.cursor + buf.len()].copy_from_slice(buf);
            self.cursor += buf.len();
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> WriterBackend for W {
    #[inline(always)]
    fn pb_write_u8(&mut self, x: u8) -> Result<()> {
        self.write_u8(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_u32(&mut self, x: u32) -> Result<()> {
        self.write_u32::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_i32(&mut self, x: i32) -> Result<()> {
        self.write_i32::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_f32(&mut self, x: f32) -> Result<()> {
        self.write_f32::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_u64(&mut self, x: u64) -> Result<()> {
        self.write_u64::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_i64(&mut self, x: i64) -> Result<()> {
        self.write_i64::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_f64(&mut self, x: f64) -> Result<()> {
        self.write_f64::<LE>(x).map_err(|e| e.into())
    }

    #[inline(always)]
    fn pb_write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.write_all(buf).map_err(|e| e.into())
    }
}
