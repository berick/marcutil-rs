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

