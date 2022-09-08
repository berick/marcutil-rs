use xml::reader::{EventReader, XmlEvent};
use std::fs::File;
use std::io::BufReader;

use super::Tag;
use super::Field;
use super::Indicator;
use super::Controlfield;
use super::Subfield;
use super::Leader;
use super::Record;

const MARCXML_NAMESPACE: &str = "http://www.loc.gov/MARC21/slim";
const MARCXML_XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";
const MARCXML_SCHEMA_LOCATION: &str =
    "http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd";

/// Replace non-ASCII characters and special characters with escaped XML entities
pub fn escape_xml(value: &str) -> String {
    let mut buf = String::new();

    for c in value.chars() {
        if c == '&' {
            buf.push_str("&amp;");
        } else if c == '\'' {
            buf.push_str("&apos;");
        } else if c == '"' {
            buf.push_str("&quot;");
        } else if c == '>' {
            buf.push_str("&gt;");
        } else if c == '<' {
            buf.push_str("&lt;");
        } else if c > '~' {
            let ord: u32 = c.into();
            buf.push_str(format!("&#x{:X};", ord).as_str());
        } else {
            buf.push(c);
        }
    }

    buf
}

struct ParseContext {
    in_cfield: bool,
    in_subfield: bool,
    in_leader: bool,
}

impl Indicator {
    pub fn to_xml(&self) -> String {
        match &self.content {
            Some(c) => c.to_string(),
            None => String::from(" "),
        }
    }
}

impl Record {

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


    /// Process a single XML read event
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
                            record.control_fields.push(Controlfield::new(&t.value)?);
                            context.in_cfield = true;

                        } else {
                            return Err(format!("Controlfield has no tag"));
                        }
                    },

                    "datafield" => {
                        let mut tag_added = false;

                        if let Some(t) =
                            attributes.iter().filter(|a| a.name.local_name.eq("tag")).next() {
                            record.fields.push(Field::new(&t.value)?);
                            tag_added = true;
                        }

                        if !tag_added { return Ok(()); }

                        if let Some(ind) =
                            attributes.iter().filter(|a| a.name.local_name.eq("ind1")).next() {
                            if let Some(mut field) = record.fields.last_mut() {
                                field.set_ind1(&ind.value)?;
                            }
                        }

                        if let Some(ind) =
                            attributes.iter().filter(|a| a.name.local_name.eq("ind2")).next() {
                            if let Some(mut field) = record.fields.last_mut() {
                                field.set_ind2(&ind.value)?;
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
        // We could use XmlWriter here, but it's overkill and
        // not quite as configurable as I'd like.

        let mut xml = String::from(r#"<?xml version="1.0"?>"#);

        // Document root

        xml += &format!(
            r#"<record xmlns="{}" xmlns:xsi="{}" xsi:schemaLocation="{}">"#,
            MARCXML_NAMESPACE, MARCXML_XSI_NAMESPACE, MARCXML_SCHEMA_LOCATION
        );

        // Leader

        xml += "<leader>";
        if let Some(ref l) = self.leader {
            xml += &escape_xml(&l.content);
        }
        xml += "</leader>";

        // Control Fields

        for cfield in &self.control_fields {
            xml += &format!(r#"<controlfield tag="{}">"#, escape_xml(&cfield.tag.content));
            if let Some(ref c) = cfield.content {
                xml += &escape_xml(c);
            }
            xml += "</controlfield>";
        }

        // Data Fields

        for field in &self.fields {

            xml += &format!(r#"<datafield tag="{}" ind1="{}" ind2="{}">"#,
                escape_xml(&field.tag.content),
                escape_xml(&field.ind1.to_xml()),
                escape_xml(&field.ind2.to_xml())
            );

            for sf in &field.subfields {
                xml += &format!(r#"<subfield code="{}">"#, sf.code);

                if let Some(ref c) = sf.content {
                    xml += &escape_xml(c);
                }

                xml += "</subfield>";
            }

            xml += "</datafield>";
        }

        xml += "</record>";

        Ok(xml)
    }
}

