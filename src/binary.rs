use std::fs::File;
use std::io::prelude::*;
use super::Record;
/*
use super::Controlfield;
use super::Field;
use super::Indicator;
use super::Leader;
use super::Subfield;
*/

const SUBFIELD_INDICATOR: u8 = 31; // '\x1F';
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

        println!("bytes: {:?}", bytes);

        match Record::from_binary(&mut bytes) {
            Ok(r) => Some(r),
            _ => None
        }
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


impl Record {

    pub fn from_binary_file(filename: &str) -> Result<BinaryRecordIterator, String> {
        BinaryRecordIterator::new(filename)
    }

    pub fn from_binary(bytes: &mut Vec<u8>) -> Result<Record, String> {
        let mut record = Record::new();
        let full_len = bytes.len();

        if full_len < RECORD_SIZE_ENTRY {
            return Err(format!("Binary record is too short"));
        }

        let size_bytes: Vec<u8> = bytes.drain(0..RECORD_SIZE_ENTRY).collect();

        let size = match std::str::from_utf8(size_bytes.as_slice()) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Error parsing size information: {e}"));
            }
        };

        let size_num = match size.parse::<usize>() {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Invalid record size: {size} {e}"));
            }
        };

        if full_len != size_num {
            return Err(format!(
                "Record has incorrect size reported={} real={}", size_num, full_len));
        }

        let leader_bytes: Vec<u8> = bytes.drain(0..LEADER_SIZE).collect();

        let leader = match std::str::from_utf8(leader_bytes.as_slice()) {
            Ok(l) => l,
            Err(e) => {
                return Err(format!(
                    "Leader value is not UTF8 compatible {:?} {}", leader_bytes, e));
            }
        };

        record.set_leader(&leader)?;

        // Process the directory entries

        Ok(record)
    }
}



