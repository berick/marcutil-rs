use std::fs::File;
use std::os::unix::prelude::FileExt;
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

pub struct BinaryRecordIterator {
    file: File,
    offset: u64,
}

impl Iterator for BinaryRecordIterator {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes: Vec<u8> = Vec::new();

        loop {
            let mut buf: [u8; 1] = [0];
            match self.file.read_at(&mut buf, self.offset) {
                Ok(count) => {
                    if count == 1 {
                        self.offset += 1;
                        bytes.push(buf[0]);
                        if buf[0] == END_OF_RECORD {
                            break;
                        }
                    }
                },
                _ => break // EOF
            }
        }

        match Record::from_binary(&bytes) {
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

        Ok(BinaryRecordIterator {
            file,
            offset: 0
        })
    }
}


impl Record {

    pub fn from_binary_file(filename: &str) -> Result<BinaryRecordIterator, String> {
        BinaryRecordIterator::new(filename)
    }


    pub fn from_binary(bytes: &Vec<u8>) -> Result<Record, String> {
        let mut record = Record::new();

        Ok(record)
    }
}



