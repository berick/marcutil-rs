use xml::reader::{EventReader, XmlEvent};
use std::fs::File;
use std::io::BufReader;
use std::fmt;

const LEADER_SIZE: usize = 24;
const TAG_SIZE: usize = 3;
const MARCXML_NAMESPACE: &'static str = "http://www.loc.gov/MARC21/slim";

#[derive(Debug, Clone)]
pub struct Tag {
    pub content: String,
}

impl Tag {

    /// Returns Err() if tag is not a 3-byte string
    pub fn new(tag: &str) -> Result<Self, String> {
        if tag.len() != TAG_SIZE {
            return Err(format!("Invalid tag: {}", tag));
        }
        Ok(Tag { content: String::from(tag) })
    }

    pub fn to_breaker(&self) -> String {
        String::from(&self.content)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Debug, Clone)]
pub struct Controlfield {
    pub tag: Tag,
    pub content: Option<String>,
}

impl Controlfield {
    pub fn new(tag: &str) -> Result<Self, String> {
        let t = Tag::new(tag)?;

        Ok(Controlfield {
            tag: t,
            content: None,
        })
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = Some(String::from(content));
    }

    pub fn to_breaker(&self) -> String {
        match &self.content {
            Some(c) => format!("{} {}", self.tag, c),
            None => format!("{}", self.tag)
        }
    }
}

impl fmt::Display for Controlfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}


#[derive(Debug, Clone)]
pub struct Subfield {
    pub code: String,
    pub content: Option<String>,
}

impl Subfield {

    pub fn new(code: &str) -> Result<Self, String> {

        if code.len() != 1 {
            return Err(format!("Invalid subfield code: {}", code));
        }

        Ok(Subfield {
            code: String::from(code),
            content: None,
        })
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = Some(String::from(content));
    }

    pub fn to_breaker(&self) -> String {
        let s = format!("${}", self.code);
        if let Some(c) = &self.content {
            s + c
        } else {
            s
        }
    }
}

impl fmt::Display for Subfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}

#[derive(Debug, Clone)]
pub enum Indicator {
    One,
    Two,
    None,
    Invalid,
}

impl Indicator {
    pub fn to_breaker(&self) -> String {
        match *self {
            Indicator::One => String::from("1"),
            Indicator::Two => String::from("2"),
            _ => String::from("\\"),
        }
    }
}

impl From<&str> for Indicator {
    fn from(value: &str) -> Self {
        match value {
            "1" => Indicator::One,
            "2" => Indicator::Two,
            ""  => Indicator::None,
            _   => Indicator::Invalid
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub tag: Tag,
    pub ind1: Indicator,
    pub ind2: Indicator,
    pub subfields: Vec<Subfield>
}

impl Field {
    pub fn new(tag: &str) -> Result<Self, String> {
        let t = Tag::new(tag)?;

        Ok(Field {
            tag: t,
            ind1: Indicator::None,
            ind2: Indicator::None,
            subfields: Vec::new()
        })
    }

    pub fn set_ind(&mut self, ind: &str, first: bool) {
        if first {
            self.ind1 = ind.into();
        } else {
            self.ind2 = ind.into();
        }
    }

    pub fn to_breaker(&self) -> String {
        let mut s = self.tag.to_breaker();
        s += self.ind1.to_breaker().as_str();
        s += self.ind2.to_breaker().as_str();

        for sf in &self.subfields {
            s += sf.to_breaker().as_str();
        }

        s
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}

#[derive(Debug, Clone)]
pub struct Leader {
    pub content: String,
}

impl Leader {

    /// Returns Err() if leader does not contain the expected number of bytes
    pub fn new(content: &str) -> Result<Self, String> {

        if content.len() != LEADER_SIZE {
            return Err(format!("Invalid leader: {}", content));
        }

        Ok(Leader { content: String::from(content) })
    }

    pub fn to_breaker(&self) -> String {
        format!("LDR {}", self.content)
    }
}

impl fmt::Display for Leader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}


#[derive(Debug, Clone)]
pub struct Record {
    pub leader: Option<Leader>,
    pub control_fields: Vec<Controlfield>,
    pub fields: Vec<Field>,
}

impl Record {

    /// Returns Err() if leader is not a 24-byte string.
    pub fn new() -> Self {
        Record {
            leader: None,
            control_fields: Vec::new(),
            fields: Vec::new(),
        }
    }

    pub fn set_leader(&mut self, leader: &str) -> Result<(), String> {
        self.leader = Some(Leader::new(leader)?);
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
        let mut record = Record::new();

        let mut in_cfield = false;
        let mut in_subfield = false;
        let mut in_leader = false;

        for evt in parser {
            match evt {

				Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    match name.local_name.as_str() {
                        "leader" => in_leader = true,
                        "controlfield" => {
                            if let Some(t) =
                                attributes.iter().filter(|a| a.name.local_name.eq("tag")).next() {
                                if let Ok(cf) = Controlfield::new(&t.value) {
                                    in_cfield = true;
                                    record.control_fields.push(cf);
                                }
                            } else {
                                return Err(format!("Controlfield has no tag"));
                            }
                        },
                        "datafield" => {
                            let mut tag_added = false;

                            if let Some(t) =
                                attributes.iter().filter(|a| a.name.local_name.eq("tag")).next() {
                                if let Ok(f) = Field::new(&t.value) {
                                    tag_added = true;
                                    record.fields.push(f);
                                }
                            }

                            if !tag_added { continue; }

                            if let Some(ind) =
                                attributes.iter().filter(|a| a.name.local_name.eq("ind1")).next() {
                                if ind.value.len() == 1 {
                                    if let Some(mut field) = record.fields.last_mut() {
                                        field.set_ind(&ind.value, true);
                                    }
                                }
                            }

                            if let Some(ind) =
                                attributes.iter().filter(|a| a.name.local_name.eq("ind2")).next() {
                                if ind.value.len() == 1 {
                                    if let Some(mut field) = record.fields.last_mut() {
                                        field.set_ind(&ind.value, false);
                                    }
                                }
                            }

                        },
                        "subfield" => {
                            if let Some(mut field) = record.fields.last_mut() {
                                if let Some(code) =
                                    attributes.iter().filter(|a| a.name.local_name.eq("code")).next() {
                                    if let Ok(sf) = Subfield::new(&code.value) {
                                        in_subfield = true;
                                        field.subfields.push(sf);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
				},

                Ok(XmlEvent::Characters(ref characters)) => {

                    if in_leader {
                        record.set_leader(characters);
                        in_leader = false;

                    } else if in_cfield {
                        if let Some(mut cf) = record.control_fields.last_mut() {
                            cf.set_content(characters);
                        }
                        in_cfield = false;

                    } else if in_subfield {
                        if let Some(mut field) = record.fields.last_mut() {
                            if let Some(mut subfield) = field.subfields.last_mut() {
                                subfield.set_content(characters);
                            }
                        }
                        in_subfield = false;
                    }
                },

				Err(e) => {
                    return Err(format!("Error parsing MARCXML: {}", e));
                }
				_ => {}
            }
        }

        Ok(record)
    }

    pub fn to_breaker(&self) -> String {
        let mut s = String::from("");

        if let Some(ref l) = self.leader {
            s += l.to_breaker().as_str();
        }

        for cfield in &self.control_fields {
            s += format!("\n{}", cfield.to_breaker()).as_str();
        }

        for field in &self.fields {
            s += format!("\n{}", field.to_breaker()).as_str();
        }

        s
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}




