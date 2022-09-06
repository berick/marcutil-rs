use xml::reader::{EventReader, XmlEvent};
use std::fs::File;
use std::io::BufReader;
use std::fmt;

const LEADER_SIZE: usize = 24;
const TAG_SIZE: usize = 3;
const MARCXML_NAMESPACE: &'static str = "http://www.loc.gov/MARC21/slim";
const PLACEHOLDER_LEADER: &'static str = "                        ";

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
}

impl fmt::Display for Controlfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            Some(c) => write!(f, "{} {}", self.tag, c),
            None => write!(f, "{}", self.tag)
        }
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

}

impl fmt::Display for Subfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.code);
        if let Some(c) = &self.content { write!(f, "{}", c); }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub tag: Tag,
    pub ind1: Option<String>,
    pub ind2: Option<String>,
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

    pub fn set_ind(&mut self, ind: &str, first: bool) -> Result<(), String> {

        if ind.eq("") || ind.eq(" ") {
            if first {
                self.ind1 = None;
            } else {
                self.ind2 = None;
            }

        } else {
            if ind.len() != 1 {
                return Err(format!("Invalid indicator value {}", ind));
            }
            if first {
                self.ind1 = Some(String::from(ind));
            } else {
                self.ind2 = Some(String::from(ind));
            }
        }

        Ok(())
    }

    //fn format_indicator
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.tag);

        match &self.ind1 {
            Some(ind) => { write!(f, "{}", ind); },
            None => { write!(f, "\\"); }
        }

        match &self.ind2 {
            Some(ind) => { write!(f, "{}", ind); },
            None => { write!(f, "\\"); }
        }

        for sf in &self.subfields {
            write!(f, "{}", sf);
        }

        Ok(())
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
}

impl fmt::Display for Leader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LDR {}", self.content)
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
        let mut record = Record::new(PLACEHOLDER_LEADER).unwrap();

        let mut cfield: Option<Controlfield> = None;
        let mut subfield: Option<Subfield> = None;
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
                                    cfield = Some(cf);
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

                    } else if cfield.is_some() {
                        let mut cf = cfield.unwrap();
                        cf.set_content(characters);
                        record.cfields.push(cf);
                        cfield = None;

                    } else {
                        // Assume we are adding field data at this point
                        // TODO is that really a safe assumption?
                        if let Some(mut field) = record.fields.last_mut() {
                            if let Some(mut subfield) = field.subfields.last_mut() {
                                subfield.set_content(characters);
                            }
                        }
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
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.leader);
        for cfield in &self.cfields {
            write!(f, "\n{}", cfield);
        }
        for field in &self.fields {
            write!(f, "\n{}", field);
        }
        Ok(())
    }
}




