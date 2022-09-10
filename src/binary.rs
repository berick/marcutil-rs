use std::fs::File;
use std::io::prelude::*;
use super::Record;
use super::Controlfield;
use super::Field;
use super::Subfield;

const SUBFIELD_SEPARATOR: &str = "\x1F";
const _END_OF_FIELD: u8 = 30; // '\x1E';
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
            // Read bytes from the file until we hit an END_OF_RECORD byte.
            // Pass the read bytes to the Record binary data reader.

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

/// bytes => String => usize
fn bytes_to_usize(bytes: &[u8]) -> Result<usize, String> {

    match std::str::from_utf8(&bytes) {
        Ok(bytes_str) => {
            match bytes_str.parse::<usize>() {
                Ok(num) => Ok(num),
                Err(e) => Err(format!("Error translating string to usize str={bytes_str} {e}"))
            }
        },
        Err(e) => Err(format!("Error translating bytes to string: {bytes:?} {e}"))
    }
}

impl Record {
    // Lets add some binary MARC data handling

    pub fn from_binary_file(filename: &str) -> Result<BinaryRecordIterator, String> {
        BinaryRecordIterator::new(filename)
    }

    pub fn from_binary(bytes: &Vec<u8>) -> Result<Record, String> {
        let mut record = Record::new();

        let rec_bytes = bytes.as_slice();
        let rec_byte_count = rec_bytes.len();

        if rec_byte_count < RECORD_SIZE_ENTRY {
            return Err(format!("Binary record is too short"));
        }

        let leader_bytes = &rec_bytes[0..LEADER_SIZE];

        // Reported size of the record
        let size_bytes = &leader_bytes[0..RECORD_SIZE_ENTRY];

        // Where in this pile of bytes do the control/data fields tart.
        let data_offset_bytes = &leader_bytes[12..17];

        let rec_size = match bytes_to_usize(&size_bytes) {
            Ok(n) => n,
            Err(e) => { return Err(e); }
        };

        if rec_byte_count != rec_size {
            return Err(format!(
                "Record has incorrect size reported={} real={}", rec_size, rec_byte_count));
        }

        record.set_leader_bytes(&leader_bytes)?;

        let data_start_idx = match bytes_to_usize(data_offset_bytes) {
            Ok(n) => n,
            Err(e) => { return Err(e); }
        };

        // -1 to skip the END_OF_FIELD
        let dir_bytes = &rec_bytes[LEADER_SIZE..(data_start_idx - 1)];

        let dir_len = dir_bytes.len();
        if dir_len == 0 || dir_len % DIRECTORY_ENTRY_LEN != 0 {
            return Err(format!("Invalid directory length {}", dir_len));
        }

        let dir_count = dir_bytes.len() / DIRECTORY_ENTRY_LEN;
        let mut dir_idx = 0;

        while dir_idx < dir_count {

            if let Err(e) =
                record.process_directory_entry(
                    &rec_bytes,
                    &dir_bytes,
                    dir_idx,
                    data_start_idx,
                    rec_byte_count)
                {
                return Err(format!(
                    "Error processing directory entry index={} {}", dir_idx, e));
            }

            dir_idx += 1;
        }

        Ok(record)
    }


    /// Unpack a single control field / data field and append to the
    /// record in progress.
    fn process_directory_entry(
        &mut self,
        rec_bytes: &[u8],
        dir_bytes: &[u8],
        dir_idx: usize,
        data_start_idx: usize,
        rec_byte_count: usize,
    ) -> Result<(), String> {

        let start = dir_idx * DIRECTORY_ENTRY_LEN;
        let end = start + DIRECTORY_ENTRY_LEN;
        let dir = &dir_bytes[start..end];

        let dir_str = match std::str::from_utf8(dir) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Invalid directory bytes: {:?} {}", dir, e));
            }
        };

        let field_tag = &dir_str[0..3];
        let field_len_str = &dir_str[3..7];
        let field_pos_str = &dir_str[7..12];

        let field_len = match field_len_str.parse::<usize>() {
            Ok(l) => l,
            Err(e) => {
                return Err(format!(
                    "Invalid data length value {} {}", field_len_str, e));
            }
        };

        // Where does this field start in the record as a whole
        let field_start_idx = match field_pos_str.parse::<usize>() {
            Ok(l) => l,
            Err(e) => {
                return Err(format!(
                    "Invalid data position value {} {}", field_pos_str, e));
            }
        };

        if (field_start_idx + field_len) > rec_byte_count {
            return Err(format!("Field length exceeds length of record for tag={field_tag}"));
        }

        let field_start = field_start_idx + data_start_idx;
        let field_end = field_start + field_len - 1; // Discard trailing END_OF_FIELD
        let field_bytes = &rec_bytes[field_start..field_end];

        let field_str = match std::str::from_utf8(&field_bytes) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!(
                    "Field data is not UTF8 compatible: {:?} {}", field_bytes, e));
            }
        };

        if field_tag < "010" { // Control field
            let mut cf = Controlfield::new(field_tag)?;
            if field_str.len() > 0 {
                cf.set_content(&field_str);
            }
            self.control_fields.push(cf);
            return Ok(());
        }

        // 3-bytes for tag
        // 1 byte for indicator 1
        // 1 byte for indicator 2
        let mut field = Field::new(field_tag).unwrap(); // tag char count is known good
        field.set_ind1(&field_str[..1]).unwrap(); // ind char count is known good
        field.set_ind2(&field_str[1..2]).unwrap(); // ind char count is known good

        // Split the remainder on the subfield separator and
        // build Field's from them.
        let field_parts: Vec<&str> = field_str.split(SUBFIELD_SEPARATOR).collect();

        for part in &field_parts[1..] { // skip the initial SUBFIELD_SEPARATOR
            let mut sf = Subfield::new(&part[..1]).unwrap(); // code size is known good
            if part.len() > 1 {
                sf.set_content(&part[1..]);
            }
            field.subfields.push(sf);
        }

        self.fields.push(field);

        Ok(())
    }
}



