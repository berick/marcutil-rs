use xml::reader::{EventReader, XmlEvent};
use std::fs::File;
use std::io::BufReader;
use std::fmt;

const LEADER_SIZE: usize = 24;
const TAG_SIZE: usize = 3;
const MARCXML_NAMESPACE: &'static str = "http://www.loc.gov/MARC21/slim";
const DEFAULT_LEADER: &'static str = "                        ";

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

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.bytes) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => {
                eprintln!("Error translating tag bytes to utf8: {:?} {}", self.bytes, e);
                Err(fmt::Error)
            }
        }
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

impl fmt::Display for Controlfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.tag, self.content)
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

impl fmt::Display for Subfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&[self.code]) {
            Ok(s) => write!(f, "${}{}", s, self.content),
            Err(e) => {
                eprintln!("Error translating subfield code to utf8: {:?} {}", self.code, e);
                Err(fmt::Error)
            }
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

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.tag);

        let ind1 = match std::str::from_utf8(&[self.ind1]) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error translating ind1 to utf8: {:?} {}", self.ind1, e);
                Err(fmt::Error)
            }
        };

        let ind1 = match std::str::from_utf8(&[self.ind1]) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error translating ind1 to utf8: {:?} {}", self.ind1, e);
                Err(fmt::Error)
            }
        };


        match std::str::from_utf8(&[self.code]) {
            Ok(s) => write!(f, "${}{}", s, self.content),
            Err(e) => {
                eprintln!("Error translating subfield code to utf8: {:?} {}", self.code, e);
                Err(fmt::Error)
            }
        }


        match self.ind1 {
            Some(i) => write!(f, "{}"
    }
}

#[derive(Debug, Clone)]
pub struct Leader {
    pub bytes: [u8; LEADER_SIZE],
}

impl Leader {

    /// Returns Err() if leader does not contain the expected number of bytes
    pub fn new(tag: &str) -> Result<Self, String> {
        let bytes = tag.as_bytes();
        if bytes.len() != LEADER_SIZE {
            return Err(format!("Invalid tag: {}", tag));
        }

        let mut lbytes: [u8; LEADER_SIZE] = [0; LEADER_SIZE];
        lbytes.copy_from_slice(&bytes[0..LEADER_SIZE]);

        Ok(Leader { bytes: lbytes })
    }
}

impl fmt::Display for Leader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.bytes) {
            Ok(s) => write!(f, "LDR {}", s),
            Err(e) => {
                eprintln!("Error translating leader bytes to utf8: {:?} {}", self.bytes, e);
                Err(fmt::Error)
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct Record {
    pub leader: Leader,
    pub cfields: Vec<Controlfield>,
    pub fields: Vec<Field>,
}

impl Record {

    /// Returns Err() if leader is not a 24-byte string.
    pub fn new(leader: &str) -> Result<Self, String> {
        let leader = Leader::new(leader)?;

        Ok(Record {
            leader,
            cfields: Vec::new(),
            fields: Vec::new(),
        })
    }

    pub fn set_leader(&mut self, leader: &str) -> Result<(), String> {
        self.leader = Leader::new(leader)?;
        Ok(())
    }

    pub fn from_xml_file(filename: &str) -> Result<Self, String> {

        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Cannot read MARCXML file: {} {}", filename, e));
            }
        };

        let file = BufReader::new(file);
        let parser = EventReader::new(file);
        let mut record = Record::new(DEFAULT_LEADER).unwrap();

        let mut cfield: Option<Controlfield> = None;
        let mut field: Option<Field> = None;
        let mut subfield: Option<Subfield> = None;
        let mut in_leader = false;

        for evt in parser {
            match evt {

				Ok(XmlEvent::StartElement { name, .. }) => {
                    match name.local_name.as_str() {
                        "leader" => in_leader = true,
                        _ => {}
                    }
				},

				Ok(XmlEvent::EndElement { name }) => {
				},

                Ok(XmlEvent::Characters(ref characters)) => {

                    if in_leader {
                        record.set_leader(characters);
                        in_leader = false;
                    }

                },

                Ok(XmlEvent::CData(characters)) => {
                    println!("CData: {}", characters);
                },

				Err(e) => {
                    return Err(format!("Error parsing MARCXML: {}", e));
                }
				_ => {}
            }
        }

        Ok(record)
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.leader)
    }
}




