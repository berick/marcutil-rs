use xml::reader::{EventReader, XmlEvent};
use std::fs;

const LEADER_SIZE: usize = 24;
const TAG_SIZE: usize = 3;

#[derive(Debug, Clone)]
pub struct Tag {
    pub bytes: [u8; TAG_SIZE],
}

impl Tag {

    /// Returns Err() if tag is not a 3-byte string
    pub fn new(tag: &str) -> Result<Self, String> {
        let tbytes = tag.as_bytes();
        if tbytes.len() != TAG_SIZE {
            return Err(format!("Invalid tag: {}", tag));
        }

        let mut tag_bytes: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag_bytes.copy_from_slice(&tbytes[0..TAG_SIZE]);
        Ok(Tag { bytes: tag_bytes })
    }
}

#[derive(Debug, Clone)]
pub struct Controlfield {
    pub tag: Tag,
    pub content: String,
}

impl Controlfield {
    pub fn new(tag: &str, content: &str) -> Result<Self, String> {
        let t = Tag::new(tag)?;
        Ok(Controlfield {
            tag: t,
            content: String::from(content),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Subfield {
    pub code: u8,
    pub content: String,
}

impl Subfield {
    pub fn new(code: u8, content: &str) -> Self {
        Subfield {
            code,
            content: String::from(content)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub tag: Tag,
    pub ind1: Option<u8>,
    pub ind2: Option<u8>,
    pub subfields: Vec<Subfield>
}

impl Field {
    pub fn new(tag: &str) -> Result<Self, String> {
        let t = Tag::new(tag)?;

        Ok(Field {
            tag: t,
            ind1: None,
            ind2: None,
            subfields: Vec::new()
        })
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub leader: [u8; LEADER_SIZE],
    pub cfields: Vec<Controlfield>,
    pub fields: Vec<Field>,
}

impl Record {

    /// Returns Err() if leader is not a 24-byte string.
    pub fn new(leader: &str) -> Result<Self, String> {

        let bytes = leader.as_bytes();

        if bytes.len() != LEADER_SIZE {
            return Err(format!("Invalid leader: {}", leader));
        }

        let mut leader_bytes: [u8; LEADER_SIZE] = [0; LEADER_SIZE];
        leader_bytes.copy_from_slice(&bytes[0..LEADER_SIZE]);

        Ok(Record {
            leader: leader_bytes,
            cfields: Vec::new(),
            fields: Vec::new(),
        })
    }

    pub fn from_xml_file(filename: &str) -> Result<Self, String> {

        let xml = match fs::read_to_string(filename) {
            Ok(x) => x,
            Err(e) => {
                return Err(format!(
                    "Cannot read MARCXML file: {} {}", filename, e));
            }
        };

        Record::from_xml(&xml)
    }

    pub fn from_xml(xml: &str) -> Result<Self, String> {
        Record::new("123123123123123123123123")
    }
}




