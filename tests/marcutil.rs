use marcutil as marc;

const MARC_XML: &str = r#"
<?xml version="1.0"?>
<record
  xmlns="http://www.loc.gov/MARC21/slim"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:schemaLocation="http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd">
  <leader>07649cim a2200913 i 4500</leader>
  <controlfield tag="001">233</controlfield>
  <controlfield tag="003">CONS</controlfield>
  <controlfield tag="005">20140128084328.0</controlfield>
  <controlfield tag="008">140128s2013    nyuopk|zqdefhi n  | ita d</controlfield>
  <datafield tag="010" ind1=" " ind2=" ">
    <subfield code="a">  2013565186</subfield>
  </datafield>
  <datafield tag="020" ind1=" " ind2=" ">
    <subfield code="a">9781480328532</subfield>
  </datafield>
  <datafield tag="020" ind1=" " ind2=" ">
    <subfield code="a">1480328537</subfield>
  </datafield>
  <datafield tag="024" ind1="1" ind2=" ">
    <subfield code="a">884088883249</subfield>
  </datafield>
  <datafield tag="028" ind1="3" ind2="2">
    <subfield code="a">HL50498721</subfield>
    <subfield code="b">Hal Leonard</subfield>
    <subfield code="q">(bk.)</subfield>
  </datafield>
</record>
"#;

#[test]
fn breaker_round_trip() {

    let record = marc::Record::from_xml(MARC_XML).expect("Created record from XML");
    let breaker = record.to_breaker();
    let record2 = marc::Record::from_breaker(&breaker).expect("Built from breaker");
    let breaker2 = record2.to_breaker();

    assert_eq!(breaker, breaker2);
}

