use getopts;
use marcutil::Record;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("x", "xml-file", "MARC XML File", "MARCXML_FILE");
    opts.optopt("b", "bin-file", "MARC Binary File", "MARC_FILE");

    let params = opts.parse(&args[1..]).expect("Options parsed");

    let xml_file_op = params.opt_str("xml-file");
    let bin_file_op = params.opt_str("bin-file");

    if xml_file_op.is_some() {
        let xml_filename = xml_file_op.unwrap();

        let mut record = Record::from_xml_file(&xml_filename).expect("MARCXML File Parse");

        if let Some(title) = record.get_values("245", "a").first() {
            println!("Maintitle => {title}");
        }

        if let Some(field) = record.get_fields_mut("245").first_mut() {
            if let Some(sf) = field.get_subfields_mut("a").first_mut() {
                sf.set_content("I Prefer This Title");
            }
        }

        if let Some(title) = record.get_values("245", "a").first() {
            println!("New Maintitle => {title}");
        }

        record
            .add_control_field("005", "123123123123")
            .expect("Added Control Field");
        record
            .add_data_field("650", vec!["a", "Hobbits", "b", "Fiction"])
            .expect("Added Data Field");

        println!("{}", record.to_xml().expect("MARC to XML OK"));

        let breaker = record.to_breaker();

        let record2 = Record::from_breaker(&breaker).expect("Create record from breaker");

        println!("\n{}", record2.to_breaker());
        println!("\n{}", record2.to_xml().expect("We made some xml"));
        println!(
            "\n{}",
            record2.to_xml_formatted().expect("We made some xml")
        );

        let bytes = record.to_binary().expect("To Binary");

        println!("{}", std::str::from_utf8(&bytes).unwrap());
    }

    if bin_file_op.is_some() {
        for record in Record::from_binary_file(&bin_file_op.unwrap()).expect("Start Binary File") {
            println!(
                "\nBinary record as xml:\n{}",
                record.to_xml_formatted().unwrap()
            );
        }
    }
}
