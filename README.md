# Rust MARC XML / Breaker Library

## Synopsis

```rs
// Parse an XML string
let record = marc::Record::from_xml(MARC_XML_STR).expect("Created record from XML");

// Turn the record into Breaker text
let breaker = record.to_breaker();

// Create a new record from previous record's breaker
let record2 = marc::Record::from_breaker(&breaker).expect("Built from breaker");

// Generate XML from our new record
let xml = record2.to_xml().expect("To XML");

println!("Generated XML: {}", xml);
```

## About

MARC Library for translating to/from MARC XML and MARC Breaker.

## Data Validation

Minimal requirements are placed on the validity and format of the data.

1. Data must be UTF-8.
1. Indicators and subfield codes must have a byte length of 1.
1. Tags must have a byte length of 3.
1. Leaders must have a byte length of 24.

In cases where these conditions are not met, routines return explanatory
Err() strings.

Otherwise, no restrictions are placed on the content of the leader, 
tag, control fields, indicators, subfield codes, or subfield values.
