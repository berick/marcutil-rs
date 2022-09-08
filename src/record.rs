const TAG_SIZE: usize = 3;
const LEADER_SIZE: usize = 24;
const INDICATOR_SIZE: usize = 1;
const SF_CODE_SIZE: usize = 1;

/// A single 3-byte tag.
#[derive(Debug, Clone)]
pub struct Tag {
    pub content: String,
}

impl Tag {
    pub fn new(tag: &str) -> Result<Self, String> {
        if tag.bytes().len() == TAG_SIZE {
            Ok(Tag {
                content: tag.to_string(),
            })
        } else {
            Err(format!("Invalid tag value: {tag}"))
        }
    }
}

/// MARC Control Field whose tag value is < "010"
#[derive(Debug, Clone)]
pub struct Controlfield {
    pub tag: Tag,
    pub content: Option<String>,
}

impl Controlfield {
    pub fn new(tag: &str) -> Result<Self, String> {
        Ok(Controlfield {
            tag: Tag::new(tag)?,
            content: None,
        })
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = Some(String::from(content));
    }
}

/// A single subfield code + value pair
#[derive(Debug, Clone)]
pub struct Subfield {
    pub code: String,
    pub content: Option<String>,
}

impl Subfield {
    pub fn new(code: &str) -> Result<Self, String> {
        if code.bytes().len() != SF_CODE_SIZE {
            return Err(format!("Invalid subfield code: {code}"));
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

/// A single 1-byte indicator value
#[derive(Debug, Clone)]
pub struct Indicator {
    pub content: Option<String>,
}

impl Indicator {
    pub fn new(value: &str) -> Result<Self, String> {
        if value.ne("") {
            // Empty indicator is fine
            if value.bytes().len() != INDICATOR_SIZE {
                return Err(format!("Invalid indicator value: '{value}'"));
            }
        }

        if value.eq("") || value.eq(" ") {
            Ok(Indicator { content: None })
        } else {
            Ok(Indicator {
                content: Some(value.to_string()),
            })
        }
    }
}

/// A MARC Data Field with tag, indicators, and subfields.
#[derive(Debug, Clone)]
pub struct Field {
    pub tag: Tag,
    pub ind1: Indicator,
    pub ind2: Indicator,
    pub subfields: Vec<Subfield>,
}

impl Field {
    pub fn new(tag: &str) -> Result<Self, String> {
        Ok(Field {
            tag: Tag::new(tag)?,
            ind1: Indicator::new("")?,
            ind2: Indicator::new("")?,
            subfields: Vec::new(),
        })
    }

    pub fn set_ind1(&mut self, ind: &str) -> Result<(), String> {
        self.set_ind(ind, true)
    }

    pub fn set_ind2(&mut self, ind: &str) -> Result<(), String> {
        self.set_ind(ind, false)
    }

    fn set_ind(&mut self, ind: &str, first: bool) -> Result<(), String> {
        let i = Indicator::new(ind)?;

        match first {
            true => self.ind1 = i,
            false => self.ind2 = i,
        }

        Ok(())
    }
    pub fn get_subfields(&self, code: &str) -> Vec<&Subfield> {
        self.subfields.iter().filter(|f| f.code.eq(code)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Leader {
    pub content: String,
}

impl Leader {
    /// Returns Err() if leader does not contain the expected number of bytes
    pub fn new(content: &str) -> Result<Self, String> {
        if content.bytes().len() != LEADER_SIZE {
            return Err(format!("Invalid leader: {content}"));
        }

        Ok(Leader {
            content: String::from(content),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub leader: Option<Leader>,
    pub control_fields: Vec<Controlfield>,
    pub fields: Vec<Field>,
}

/// A MARC record with leader, control fields, and data fields.
impl Record {
    pub fn new() -> Self {
        Record {
            leader: None,
            control_fields: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Apply a leader value
    ///
    /// Returns Err if the value is not composed of the correct number
    /// of bytes.
    pub fn set_leader(&mut self, leader: &str) -> Result<(), String> {
        self.leader = Some(Leader::new(leader)?);
        Ok(())
    }

    pub fn get_control_fields(&self, tag: &str) -> Vec<&Controlfield> {
        self.control_fields
            .iter()
            .filter(|f| f.tag.content.eq(tag))
            .collect()
    }

    pub fn get_fields(&self, tag: &str) -> Vec<&Field> {
        self.fields
            .iter()
            .filter(|f| f.tag.content.eq(tag))
            .collect()
    }

    pub fn get_values(&self, tag: &str, sfcode: &str) -> Vec<&str> {
        let mut vec = Vec::new();
        for field in self.get_fields(tag) {
            for sf in field.get_subfields(sfcode) {
                if let Some(content) = &sf.content {
                    vec.push(content.as_str());
                }
            }
        }
        vec
    }
}
