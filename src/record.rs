const TAG_SIZE: usize = 3;
const LEADER_SIZE: usize = 24;
const INDICATOR_SIZE: usize = 1;

#[derive(Debug, Clone)]
pub struct Tag {
    pub content: String,
}

impl Tag {
    pub fn new(tag: &str) -> Result<Self, String> {
        let bytes = tag.as_bytes();
        if tag.bytes().len() == TAG_SIZE {
            Ok(Tag { content: tag.to_string() })
        } else {
            Err(format!("Invalid tag value: {}", tag))
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Indicator {
    pub content: Option<String>,
}

impl Indicator {
    pub fn new(value: &str) -> Result<Self, String> {

        if value.len() > 0 { // Empty indicator is fine
            if value.bytes().len() != INDICATOR_SIZE {
                return Err(format!("Invalid indicator value: '{}'", value));
            }
        }

        if value.eq("") || value.eq(" ") {
            Ok(Indicator { content: None })
        } else {
            Ok(Indicator { content: Some(value.to_string()) })
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
        Ok(Field {
            tag: Tag::new(tag)?,
            ind1: Indicator::new("")?,
            ind2: Indicator::new("")?,
            subfields: Vec::new()
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
}

#[derive(Debug, Clone)]
pub struct Leader {
    pub content: String,
}

impl Leader {

    /// Returns Err() if leader does not contain the expected number of bytes
    pub fn new(content: &str) -> Result<Self, String> {

        if content.as_bytes().len() != LEADER_SIZE {
            return Err(format!("Invalid leader: {}", content));
        }

        Ok(Leader { content: String::from(content) })
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub leader: Option<Leader>,
    pub control_fields: Vec<Controlfield>,
    pub fields: Vec<Field>,
}

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

}

