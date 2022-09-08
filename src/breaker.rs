use super::Tag;
use super::Field;
use super::Indicator;
use super::Controlfield;
use super::Subfield;
use super::Leader;
use super::Record;

const MARC_BREAKER_SF_DELIMITER: &str = "$";
const MARC_BREAKER_SF_DELIMITER_ESCAPE: &str = "{dollar}";

pub fn escape_to_breaker(value: &str) -> String {
    value.replace(MARC_BREAKER_SF_DELIMITER, MARC_BREAKER_SF_DELIMITER_ESCAPE)
}

pub fn unescape_from_breaker(value: &str) -> String {
    value.replace(MARC_BREAKER_SF_DELIMITER_ESCAPE, MARC_BREAKER_SF_DELIMITER)
}


impl Controlfield {
    pub fn to_breaker(&self) -> String {
        match &self.content {
            Some(c) => format!("{} {}", self.tag.content, escape_to_breaker(c)),
            None => format!("{}", self.tag.content)
        }
    }
}

impl Subfield {
    pub fn to_breaker(&self) -> String {
        let s = format!("${}", self.code);
        if let Some(c) = &self.content {
            s + escape_to_breaker(c).as_str()
        } else {
            s
        }
    }
}

impl Indicator {
    pub fn to_breaker(&self) -> String {
        match &self.content {
            Some(c) => c.to_string(),
            None => String::from("\\"),
        }
    }
}

impl Field {
    pub fn to_breaker(&self) -> String {

        let mut s = format!("{} {}{}",
            self.tag.content,
            self.ind1.to_breaker(),
            self.ind2.to_breaker()
        );

        for sf in &self.subfields {
            s += sf.to_breaker().as_str();
        }

        s
    }
}

impl Leader {
    pub fn to_breaker(&self) -> String {
        format!("LDR {}", escape_to_breaker(&self.content))
    }
}

impl Record {
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

    pub fn from_breaker(breaker: &str) -> Result<Self, String> {

        let mut record = Record::new();

        for line in breaker.lines() {
            record.add_breaker_line(line)?;
        }

        Ok(record)
    }

    /// Process one line of breaker text
    fn add_breaker_line(&mut self, line: &str) -> Result<(), String> {
        let len = line.len();

        if len < 3 { return Ok(()); }

        let tag = &line[..3];

        if tag.eq("LDR") {
            if len > 4 {
                self.set_leader(&line[4..])?;
            }
            return Ok(());
        }

        if tag < "010" {

            let mut cf = Controlfield::new(tag)?;
            if len > 4 {
                cf.set_content(unescape_from_breaker(&line[4..]).as_str());
            }
            self.control_fields.push(cf);
            return Ok(());
        }

        let mut field = Field::new(tag)?;

        if len > 4 {
            field.set_ind1(&line[4..5]);
        }

        if len > 5 {
            field.set_ind2(&line[5..6]);
        }

        if len > 6 {
            for sf in line[6..].split(MARC_BREAKER_SF_DELIMITER) {
                if sf.len() == 0 { continue; }
                let mut subfield = Subfield::new(&sf[..1])?;
                if sf.len() > 1 {
                    subfield.set_content(unescape_from_breaker(&sf[1..]).as_str());
                }
                field.subfields.push(subfield);
            }
        }

        self.fields.push(field);

        Ok(())
    }
}
