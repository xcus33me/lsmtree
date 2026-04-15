// Block-based implementation approach (rocksdb-like).
// We write to a 32kiB buffer, and when the end is reached, we flush the writes to disk

use std::{fs::File, io::Write, path::PathBuf};

const BUF_SIZE: usize = 32768; // 32KiB
const HEADER_SIZE: usize = 7; // CRC(4) + Length(2) + Type(1)

// One logical record may not fit into the rest of the block and is split into fragments.
// "FULL"   = The entire logical record fits into the current block;
// "FIRST"  = The recording did not fit, this is the beginning of it;
// "MIDDLE" = The middle fragment of the record. Appreas only if the record is soo large that it spans three of more blocks;
// "Last"   = The end of multi-fragment record;
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum RecordType {
    Full = 0x01,
    First = 0x02,
    Middle = 0x03,
    Last = 0x04,
}

pub struct Wal {
    writer: Writer,
}

struct Writer {
    path: PathBuf,
    file: File,

    buf: [u8; BUF_SIZE],
    buf_pos: usize,

    is_dirty: bool,
}

impl Writer {
    fn write_fragment(&mut self, record_type: RecordType, record: &[u8]) {
        // create Crc32 bytes
        let mut hasher = crc32fast::Hasher::new();

        hasher.update(&[record_type as u8]);
        hasher.update(record);

        let crc = hasher.finalize();

        self.buf[self.buf_pos..self.buf_pos + 4].copy_from_slice(&crc.to_le_bytes());
        self.buf_pos += 4;

        // create Length bytes
        let len = record.len() as u16;
        self.buf[self.buf_pos..self.buf_pos + 2].copy_from_slice(&len.to_le_bytes());
        self.buf_pos += 2;

        // create Type bytes
        self.buf[self.buf_pos] = record_type as u8;
        self.buf_pos += 1;

        // create Data bytes
        self.buf[self.buf_pos..self.buf_pos + record.len()].copy_from_slice(record);
        self.buf_pos += record.len();
    }

    fn write_record(&mut self, record: &[u8]) -> Result<(), crate::Error> {
        let mut offset = 0;
        let mut first = true;

        loop {
            let remaining = BUF_SIZE - self.buf_pos;

            // if the remaining space in the block is not enough even for the header - fill it with zeroes.
            if remaining < HEADER_SIZE {
                self.buf[self.buf_pos..BUF_SIZE].fill(0);
                self.buf_pos = BUF_SIZE;
                self.flush_buffer()?;
            }

            let remaining = BUF_SIZE - self.buf_pos; // recalculate after a possible flush
            let available_data = remaining - HEADER_SIZE;
            let leftover = record.len() - offset;

            if leftover <= available_data {
                let record_type = if first {
                    RecordType::Full
                } else {
                    RecordType::Last
                };

                self.write_fragment(record_type, &record[offset..]);
                self.is_dirty = true;
                break;
            } else {
                let record_type = if first {
                    RecordType::First
                } else {
                    RecordType::Middle
                };

                self.write_fragment(record_type, &record[offset..offset + available_data]);
                self.is_dirty = true;
                offset += available_data;
                first = false;
                self.flush_buffer()?;
            }
        }

        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), crate::Error> {
        if self.buf_pos == 0 {
            return Ok(());
        }

        self.file.write_all(&self.buf[0..self.buf_pos])?;
        self.buf_pos = 0;
        self.is_dirty = false;
        Ok(())
    }
}

struct Reader {
    buf: [u8; BUF_SIZE],
    buf_pos: usize,
    buf_len: usize, // how much has actually been read (the last block may be incomplete)
}
