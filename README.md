# Rust MARC XML / Breaker / Binary Library

## Synopsis

```rs
use marcutil::Record;

// Parse an XML string
let record = Record::from_xml(MARC_XML_STR).expect("Created record from XML");

if let Some(title) = record.get_values("245", "a").first() {
    println!("Maintitle => {title}");
}

// Modify a field value
if let Some(field) = record.get_fields_mut("245").first_mut() {
    if let Some(sf) = field.get_subfields_mut("a").first_mut() {
        sf.set_content("I Prefer This Title");
    }
}

if let Some(title) = record.get_values("245", "a").first() {
    println!("New Maintitle => {title}");
}

// Turn the record into Breaker text
let breaker = record.to_breaker();

println!("Breaker: {breaker}");

// Create a new record from previous record's breaker
let record2 = Record::from_breaker(&breaker).expect("Built from breaker");

// Generate XML from our new record
let xml = record2.to_xml().expect("To XML");

println!("Generated XML: {xml}");

// Binary file reading
for rec in Record::from_binary_file(MARC_FILENAME).expect("Start Binary File") {
    println!("\nBinary record as xml:\n{}", rec.to_xml_formatted().unwrap());
} 

```

## About

MARC Library for translating to/from MARC XML and MARC Breaker.

## Data Validation

Minimal requirements are placed on the validity and format of the data.

1. Data must be UTF-8 compatible.
1. Indicators and subfield codes must have a byte length of 1.
1. Tags must have a byte length of 3.
1. Leaders must have a byte length of 24.
1. Control fields and data fields must have a tag.
1. Binary leader/directory metadata must be sane.

In cases where these conditions are not met, routines exit early with
explanatory Err() strings.

Otherwise, no restrictions are placed on the data values.
