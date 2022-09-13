use std::fs::File;
use xml::reader::{EventReader, XmlEvent};

use super::Controlfield;
use super::Field;
use super::Record;
use super::Subfield;

const MARCXML_NAMESPACE: &str = "http://www.loc.gov/MARC21/slim";
const MARCXML_XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";
const MARCXML_SCHEMA_LOCATION: &str =
    "http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd";

/// Replace non-ASCII characters and special characters with escaped
/// XML entities.
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
            buf.push_str(format!("&#x{ord:X};").as_str());
        } else {
            buf.push(c);
        }
    }

    buf
}

fn format(formatted: bool, value: &mut String, depth: u8) {
    if formatted {
        value.push_str("\n");
        for _ in 0..depth {
            value.push_str(" ");
        }
    }
}

struct XmlParseContext {
    in_cfield: bool,
    in_subfield: bool,
    in_leader: bool,
    record_complete: bool,
}

pub struct XmlRecordIterator {
    reader: Option<EventReader<File>>,
    string: Option<String>,
}

impl Iterator for XmlRecordIterator {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let mut context = XmlParseContext {
            in_cfield: false,
            in_subfield: false,
            in_leader: false,
            record_complete: false,
        };

        if self.reader.is_some() {
            self.read_next_from_file(&mut context)
        } else {
            self.read_next_from_string(&mut context)
        }
    }
}

impl XmlRecordIterator {
    pub fn from_file(filename: &str) -> Result<Self, String> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Cannot read MARCXML file: {filename} {e}"));
            }
        };

        Ok(XmlRecordIterator {
            string: None,
            reader: Some(EventReader::new(file)),
        })
    }

    pub fn from_string(xml: &str) -> Result<Self, String> {
        Ok(XmlRecordIterator {
            string: Some(xml.to_string()),
            reader: None,
        })
    }

    fn read_next_from_string(&mut self, context: &mut XmlParseContext) -> Option<Record> {
        let mut record = Record::new();
        None
    }

    fn read_next_from_file(&mut self, context: &mut XmlParseContext) -> Option<Record> {
        let mut record = Record::new();

        let reader = match &mut self.reader {
            Some(r) => r,
            None => {
                return None;
            }
        };

        loop {
            match reader.next() {
                Ok(evt) => {
                    if XmlEvent::EndDocument == evt {
                        // All done.
                        return None;
                    }

                    match Record::handle_xml_read_event(&mut record, context, evt) {
                        Ok(_) => {
                            if context.record_complete {
                                return Some(record);
                            }
                        }
                        Err(e) => {
                            // Can't return an Err() from an iterator, so
                            // log the issue and carry on.
                            eprintln!("Error processing XML: {e}");
                            return None;
                        }
                    }
                }
                Err(e) => {
                    // Can't return an Err() from an iterator, so
                    // log the issue and carry on.
                    eprintln!("Error processing XML: {e}");
                    return None;
                }
            }
        }
    }
}

impl Record {
    /// Returns an iterator over the XML file which emits Records.
    pub fn from_xml_file(filename: &str) -> Result<XmlRecordIterator, String> {
        Ok(XmlRecordIterator::from_file(filename)?)
    }

    /// TODO ITERATOR
    /// Returns a single Record from the XML.
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let parser = EventReader::new(xml.as_bytes());
        let mut record = Record::new();

        let mut context = XmlParseContext {
            in_cfield: false,
            in_subfield: false,
            in_leader: false,
            record_complete: false,
        };

        for evt_res in parser {
            match evt_res {
                Ok(evt) => {
                    Record::handle_xml_read_event(&mut record, &mut context, evt)?;
                }
                Err(e) => {
                    return Err(format!("Error parsing MARCXML: {e}"));
                }
            }

            if context.record_complete {
                // In case there are multiple records in the file.
                // We just want the first.
                break;
            }
        }

        Ok(record)
    }

    /// Process a single XML read event
    fn handle_xml_read_event(
        record: &mut Record,
        context: &mut XmlParseContext,
        evt: XmlEvent,
    ) -> Result<(), String> {
        match evt {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "leader" => {
                    context.in_leader = true;
                }

                "controlfield" => {
                    if let Some(t) = attributes
                        .iter()
                        .filter(|a| a.name.local_name.eq("tag"))
                        .next()
                    {
                        record
                            .control_fields
                            .push(Controlfield::new(&t.value, None)?);
                        context.in_cfield = true;
                    } else {
                        return Err(format!("Controlfield has no tag"));
                    }
                }

                "datafield" => {
                    if let Some(t) = attributes
                        .iter()
                        .filter(|a| a.name.local_name.eq("tag"))
                        .next()
                    {
                        record.fields.push(Field::new(&t.value)?);
                    } else {
                        return Err(format!("Data field has no tag"));
                    }

                    if let Some(ind) = attributes
                        .iter()
                        .filter(|a| a.name.local_name.eq("ind1"))
                        .next()
                    {
                        if let Some(field) = record.fields.last_mut() {
                            field.set_ind1(&ind.value)?;
                        }
                    }

                    if let Some(ind) = attributes
                        .iter()
                        .filter(|a| a.name.local_name.eq("ind2"))
                        .next()
                    {
                        if let Some(field) = record.fields.last_mut() {
                            field.set_ind2(&ind.value)?;
                        }
                    }
                }

                "subfield" => {
                    if let Some(field) = record.fields.last_mut() {
                        if let Some(code) = attributes
                            .iter()
                            .filter(|a| a.name.local_name.eq("code"))
                            .next()
                        {
                            if let Ok(sf) = Subfield::new(&code.value, None) {
                                context.in_subfield = true;
                                field.subfields.push(sf);
                            }
                        }
                    }
                }
                _ => {}
            },

            XmlEvent::Characters(ref characters) => {
                if context.in_leader {
                    record.set_leader(characters)?;
                    context.in_leader = false;
                } else if context.in_cfield {
                    if let Some(cf) = record.control_fields.last_mut() {
                        cf.set_content(characters);
                    }
                    context.in_cfield = false;
                } else if context.in_subfield {
                    if let Some(field) = record.fields.last_mut() {
                        if let Some(subfield) = field.subfields.last_mut() {
                            subfield.set_content(characters);
                        }
                    }
                    context.in_subfield = false;
                }
            }

            XmlEvent::EndElement { name, .. } => match name.local_name.as_str() {
                "record" => context.record_complete = true,
                _ => {}
            },

            _ => {}
        }

        Ok(())
    }

    /// Creates the XML representation of a MARC record as a String.
    pub fn to_xml(&self) -> Result<String, String> {
        self.to_xml_shared(false)
    }

    /// Creates the XML representation of a MARC record as a formatted
    /// string using 2-space indentation.
    pub fn to_xml_formatted(&self) -> Result<String, String> {
        self.to_xml_shared(true)
    }

    fn to_xml_shared(&self, formatted: bool) -> Result<String, String> {
        // We could use XmlWriter here, but it's overkill and
        // not quite as configurable as I'd like.

        let mut xml = String::from(r#"<?xml version="1.0"?>"#);

        // Document root

        if formatted {
            xml += &format!(
                "\n<record\n  xmlns=\"{}\"\n  xmlns:xsi=\"{}\"\n  xsi:schemaLocation=\"{}\">",
                MARCXML_NAMESPACE, MARCXML_XSI_NAMESPACE, MARCXML_SCHEMA_LOCATION
            );
        } else {
            xml += &format!(
                r#"<record xmlns="{}" xmlns:xsi="{}" xsi:schemaLocation="{}">"#,
                MARCXML_NAMESPACE, MARCXML_XSI_NAMESPACE, MARCXML_SCHEMA_LOCATION
            );
        }

        // Leader

        format(formatted, &mut xml, 2);

        xml += "<leader>";
        if let Some(ref l) = self.leader {
            xml += &escape_xml(&l.content);
        }
        xml += "</leader>";

        // Control Fields

        for cfield in &self.control_fields {
            format(formatted, &mut xml, 2);

            xml += &format!(
                r#"<controlfield tag="{}">{}</controlfield>"#,
                escape_xml(&cfield.tag.content),
                escape_xml(&cfield.content),
            );
        }

        // Data Fields

        for field in &self.fields {
            format(formatted, &mut xml, 2);

            xml += &format!(
                r#"<datafield tag="{}" ind1="{}" ind2="{}">"#,
                escape_xml(&field.tag.content),
                escape_xml(&field.ind1.to_string()),
                escape_xml(&field.ind2.to_string())
            );

            for sf in &field.subfields {
                format(formatted, &mut xml, 4);

                xml += &format!(
                    r#"<subfield code="{}">{}</subfield>"#,
                    &escape_xml(&sf.code),
                    &escape_xml(&sf.content)
                );
            }

            format(formatted, &mut xml, 2);

            xml += "</datafield>";
        }

        format(formatted, &mut xml, 0);

        xml += "</record>";

        Ok(xml)
    }
}
