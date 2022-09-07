use xml::reader::{EventReader, XmlEvent};
use xml::writer::{EmitterConfig, EventWriter};
use xml::writer::XmlEvent as WriteEvent;
use std::fs::File;
use std::io::BufReader;
use std::fmt;

const TAG_SIZE: usize = 3;
const LEADER_SIZE: usize = 24;
const MARCXML_NAMESPACE: &'static str = "http://www.loc.gov/MARC21/slim";
const MARCXML_XSI_NAMESPACE: &'static str = "http://www.w3.org/2001/XMLSchema-instance";
const MARCXML_SCHEMA_LOCATION: &'static str =
    "http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd";

fn escape_breaker(value: &str) -> String {
    value.replace("$", "${dollar}")
}

/// Replace non-ASCII characters with escaped XML entities
fn escape_to_ascii(value: &str) -> String {

    let mut s = String::new();

    for c in value.chars() {
        let ord: u32 = c.into();
        if ord > 126 {
            s.push_str(format!("&#x{:X};", ord).as_str());
        } else {
            s.push(c);
        }
    }

    s
}

#[derive(Debug, Clone)]
pub struct Controlfield {
    pub tag: String,
    pub content: Option<String>,
}

impl Controlfield {
    pub fn new(tag: &str) -> Self {
        Controlfield {
            tag: tag.to_string(),
            content: None,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = Some(String::from(content));
    }

    pub fn to_breaker(&self) -> String {
        match &self.content {
            Some(c) => format!("{} {}", self.tag, escape_breaker(c)),
            None => format!("{}", self.tag)
        }
    }
}

impl fmt::Display for Controlfield {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            s + escape_breaker(c).as_str()
        } else {
            s
        }
    }
}

impl fmt::Display for Subfield {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub tag: String,
    pub ind1: Option<String>,
    pub ind2: Option<String>,
    pub subfields: Vec<Subfield>
}

impl Field {
    pub fn new(tag: &str) -> Self {
        Field {
            tag: tag.to_string(),
            ind1: None,
            ind2: None,
            subfields: Vec::new()
        }
    }

    pub fn set_ind1(&mut self, ind: &str) {
        self.set_ind(ind, true);
    }

    pub fn set_ind2(&mut self, ind: &str) {
        self.set_ind(ind, false);
    }

    fn set_ind(&mut self, ind: &str, first: bool) {
        if first {
            self.ind1 = Some(ind.to_string());
        } else {
            self.ind2 = Some(ind.to_string());
        }
    }

    pub fn to_breaker(&self) -> String {

        let mut s = format!("{} ", self.tag);

        match &self.ind1 {
            Some(i) => s += format!("{i}").as_str(),
            None => s += "\\"
        }

        match &self.ind2 {
            Some(i) => s += format!("{i}").as_str(),
            None => s += "\\"
        }

        for sf in &self.subfields {
            s += sf.to_breaker().as_str();
        }

        s
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        format!("LDR {}", escape_breaker(&self.content))
    }
}

impl fmt::Display for Leader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}

struct ParseContext {
    in_cfield: bool,
    in_subfield: bool,
    in_leader: bool,
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

    /// Creates a Record from an XML file
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

        let mut context = ParseContext {
            in_cfield: false,
            in_subfield: false,
            in_leader: false,
        };

        for evt_res in parser {
            match evt_res {
                Ok(evt) => {
                    Record::handle_xml_read_event(&mut record, &mut context, evt)?;
                },
                Err(e) => {
                    return Err(format!("Error parsing MARCXML: {}", e));
                }
            }
        }

        Ok(record)
    }

    /// Creates a Record from an XML string
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let parser = EventReader::new(xml.as_bytes());
        let mut record = Record::new();

        let mut context = ParseContext {
            in_cfield: false,
            in_subfield: false,
            in_leader: false,
        };

        for evt_res in parser {
            match evt_res {
                Ok(evt) => {
                    Record::handle_xml_read_event(&mut record, &mut context, evt)?;
                },
                Err(e) => {
                    return Err(format!("Error parsing MARCXML: {}", e));
                }
            }
        }

        Ok(record)
    }


    fn handle_xml_read_event(record: &mut Record,
        context: &mut ParseContext, evt: XmlEvent) -> Result<(), String> {

        match evt {

            XmlEvent::StartElement { name, attributes, .. } => {
                match name.local_name.as_str() {

                    "leader" => {
                        context.in_leader = true;
                    },

                    "controlfield" => {
                        if let Some(t) =
                            attributes.iter().filter(|a| a.name.local_name.eq("tag")).next() {
                            record.control_fields.push(Controlfield::new(&t.value));
                            context.in_cfield = true;

                        } else {
                            return Err(format!("Controlfield has no tag"));
                        }
                    },

                    "datafield" => {
                        let mut tag_added = false;

                        if let Some(t) =
                            attributes.iter().filter(|a| a.name.local_name.eq("tag")).next() {
                            record.fields.push(Field::new(&t.value));
                            tag_added = true;
                        }

                        if !tag_added { return Ok(()); }

                        if let Some(ind) =
                            attributes.iter().filter(|a| a.name.local_name.eq("ind1")).next() {
                            if let Some(mut field) = record.fields.last_mut() {
                                field.set_ind1(&ind.value);
                            }
                        }

                        if let Some(ind) =
                            attributes.iter().filter(|a| a.name.local_name.eq("ind2")).next() {
                            if let Some(mut field) = record.fields.last_mut() {
                                field.set_ind2(&ind.value);
                            }
                        }
                    },

                    "subfield" => {
                        if let Some(mut field) = record.fields.last_mut() {
                            if let Some(code) =
                                attributes.iter().filter(|a| a.name.local_name.eq("code")).next() {
                                if let Ok(sf) = Subfield::new(&code.value) {
                                    context.in_subfield = true;
                                    field.subfields.push(sf);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            },

            XmlEvent::Characters(ref characters) => {

                if context.in_leader {
                    record.set_leader(characters);
                    context.in_leader = false;

                } else if context.in_cfield {
                    if let Some(mut cf) = record.control_fields.last_mut() {
                        cf.set_content(characters);
                    }
                    context.in_cfield = false;

                } else if context.in_subfield {
                    if let Some(mut field) = record.fields.last_mut() {
                        if let Some(mut subfield) = field.subfields.last_mut() {
                            subfield.set_content(characters);
                        }
                    }
                    context.in_subfield = false;
                }
            },
            _ => {}
        }

        Ok(())
    }

    pub fn to_xml(&self) -> Result<String, String> {

        let mut dest: Vec<u8> = Vec::new();
        let mut writer = EmitterConfig::new().create_writer(&mut dest);

        let root_event =
            WriteEvent::start_element("record")
            .attr("xmlns", MARCXML_NAMESPACE)
            .attr("xmlns:xsi", MARCXML_XSI_NAMESPACE)
            .attr("xsi:schemaLocation", MARCXML_SCHEMA_LOCATION);

        writer.write(root_event);

        // Leader
        writer.write(WriteEvent::start_element("leader"));
        if let Some(ref l) = self.leader {
            writer.write(WriteEvent::characters(&l.content));
        }
        writer.write(WriteEvent::end_element());

        // Controlfields
        for cfield in &self.control_fields {
            writer.write(
                WriteEvent::start_element("controlfield")
                .attr("tag", &cfield.tag)
            );
            if let Some(ref c) = cfield.content {
                writer.write(WriteEvent::characters(c));
            }
            writer.write(WriteEvent::end_element());
        }

        for field in &self.fields {

            let ind1 = match &field.ind1 {
                Some(ref v) => v,
                None => ""
            };

            let ind2 = match &field.ind2 {
                Some(ref v) => v,
                None => ""
            };

            writer.write(
                WriteEvent::start_element("datafield")
                .attr("tag", &field.tag)
                .attr("ind1", ind1)
                .attr("ind2", ind2)
            );

            for sf in &field.subfields {
                writer.write(
                    WriteEvent::start_element("subfield")
                    .attr("code", &sf.code)
                );

                if let Some(ref c) = sf.content {
                    writer.write(WriteEvent::characters(c));
                }

                // End Subfield
                writer.write(WriteEvent::end_element());
            }

            // End Datafield
            writer.write(WriteEvent::end_element());
        }

        // End root element
        writer.write(WriteEvent::end_element());

        match std::str::from_utf8(&dest) {
            Ok(s) => Ok(escape_to_ascii(s)),
            Err(e) => Err(format!(
                "Error converting MARC bytes to string: {:?} {}", dest, e))
        }
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_breaker())
    }
}


