use std::fs::File;
use std::io::prelude::*;
use super::Record;
use super::Controlfield;
use super::Field;
use super::Subfield;

const SUBFIELD_INDICATOR: &str = "\x1F";
const END_OF_FIELD: u8 = 30; // '\x1E';
const END_OF_RECORD: u8 = 29; // '\x1D';
const DIRECTORY_ENTRY_LEN: usize = 12;
const RECORD_SIZE_ENTRY: usize = 5;
const LEADER_SIZE: usize = 24;

pub struct BinaryRecordIterator {
    file: File,
}

impl Iterator for BinaryRecordIterator {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes: Vec<u8> = Vec::new();

        loop {
            let mut buf: [u8; 1] = [0];
            match self.file.read(&mut buf) {
                Ok(count) => {
                    if count == 1 {
                        bytes.push(buf[0]);
                        if buf[0] == END_OF_RECORD {
                            break;
                        }
                    } else {
                        break; // EOF
                    }
                },
                Err(e) => {
                    // Can't really return an Err from an Iterator.
                    // Log the error and wrap it up.
                    eprintln!("Error reading file: {:?} {}", self.file, e);
                    break;
                }
            }
        }

        if bytes.len() > 0 {
            match Record::from_binary(&bytes) {
                Ok(r) => {
                    return Some(r);
                },
                Err(e) => {
                    eprintln!("Error processing bytes: {:?} {}", bytes, e);
                    return None;
                }
            }
        }

        None
    }
}

impl BinaryRecordIterator {

    pub fn new(filename: &str) -> Result<Self, String> {

        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Cannot read MARC file: {filename} {e}"));
            }
        };

        Ok(BinaryRecordIterator { file })
    }
}

fn bytes_to_usize(bytes: &[u8]) -> Result<usize, String> {

    if let Ok(bytes_str) = std::str::from_utf8(&bytes) {
        if let Ok(bytes_num) = bytes_str.parse::<usize>() {
            return Ok(bytes_num);
        }
    }

    Err(format!("Invalid byte sequence for number: {:?}", bytes))
}


impl Record {

    pub fn from_binary_file(filename: &str) -> Result<BinaryRecordIterator, String> {
        BinaryRecordIterator::new(filename)
    }

    pub fn from_binary(bytes: &Vec<u8>) -> Result<Record, String> {
        let mut record = Record::new();
        let bytes = bytes.as_slice();
        let full_len = bytes.len();

        if full_len < RECORD_SIZE_ENTRY {
            return Err(format!("Binary record is too short"));
        }

        let leader_bytes = &bytes[0..LEADER_SIZE];
        let size_bytes = &leader_bytes[0..RECORD_SIZE_ENTRY];
        let offset_bytes = &leader_bytes[12..17];

        let rec_size = match bytes_to_usize(&size_bytes) {
            Ok(n) => n,
            Err(e) => { return Err(e); }
        };

        if full_len != rec_size {
            return Err(format!(
                "Record has incorrect size reported={} real={}", rec_size, full_len));
        }

        let leader = match std::str::from_utf8(&leader_bytes) {
            Ok(l) => l,
            Err(e) => {
                return Err(format!(
                    "Leader value is not UTF8 compatible {:?} {}", leader_bytes, e));
            }
        };

        record.set_leader(&leader)?;

        // position 12 - 16 of the leader give offset to the body

        let body_start_pos = match bytes_to_usize(offset_bytes) {
            Ok(n) => n,
            Err(e) => { return Err(e); }
        };

        // -1 to skip the END_OF_FIELD
        let dir_bytes = &bytes[LEADER_SIZE..(body_start_pos - 1)];

        let dir_len = dir_bytes.len();
        if dir_len == 0 || dir_len % DIRECTORY_ENTRY_LEN != 0 {
            return Err(format!("Invalid directory length {}", dir_len));
        }

        let dir_count = dir_bytes.len() / DIRECTORY_ENTRY_LEN;
        let mut dir_idx = 0;

        while dir_idx < dir_count {

            let start = dir_idx * DIRECTORY_ENTRY_LEN;
            let end = start + DIRECTORY_ENTRY_LEN;
            let dir = &dir_bytes[start..end];

            dir_idx += 1;

            let dir_str = match std::str::from_utf8(dir) {
                Ok(s) => s,
                Err(e) => {
                    return Err(format!("Invalid directory bytes: {:?} {}", dir, e));
                }
            };

            let tag = &dir_str[0..3];
            let len_str = &dir_str[3..7];
            let pos_str = &dir_str[7..12];

            let len = match len_str.parse::<usize>() {
                Ok(l) => l,
                Err(e) => {
                    return Err(format!(
                        "Invalid data length value {} {}", len_str, e));
                }
            };

            let pos = match pos_str.parse::<usize>() {
                Ok(l) => l,
                Err(e) => {
                    return Err(format!(
                        "Invalid data position value {} {}", pos_str, e));
                }
            };

            if (pos + len) > full_len {
                return Err(format!("Field length exceeds length of record for tag={tag}"));
            }

            let dstart = body_start_pos + pos;
            let dend = dstart + len - 1; // Drop trailing END_OF_FIELD
            let field_bytes = &bytes[dstart..dend];
            let field_str = match std::str::from_utf8(&field_bytes) {
                Ok(s) => s,
                Err(e) => {
                    return Err(format!(
                        "Field data is not UTF8 compatible: {:?} {}", field_bytes, e));
                }
            };

            if tag < "010" {
                let mut cf = Controlfield::new(tag)?;
                if field_str.len() > 0 {
                    cf.set_content(&field_str);
                }
                record.control_fields.push(cf);
                continue;
            }

            let mut field = Field::new(tag).unwrap(); // tag char count is known good
            field.set_ind1(&field_str[..1]).unwrap(); // ind char count is known good
            field.set_ind2(&field_str[1..2]).unwrap(); // ind char count is known good

            let field_parts: Vec<&str> = field_str.split(SUBFIELD_INDICATOR).collect();

            for part in &field_parts[1..] {
                let mut sf = Subfield::new(&part[..1]).unwrap(); // code size is known good
                if part.len() > 1 {
                    sf.set_content(&part[1..]);
                }
                field.subfields.push(sf);
            }

            record.fields.push(field);
        }

        Ok(record)
    }
}



