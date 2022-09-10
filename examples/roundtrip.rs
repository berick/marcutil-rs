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

        let record = Record::from_xml_file(&xml_filename).expect("MARCXML File Parse");

        if let Some(title) = record.get_values("245", "a").first() {
            println!("Maintitle => {title}");
        }

        println!("{}", record.to_xml().expect("MARC to XML OK"));

        let breaker = record.to_breaker();

        let record2 = Record::from_breaker(&breaker).expect("Create record from breaker");

        println!("\n{}", record2.to_breaker());
        println!("\n{}", record2.to_xml().expect("We made some xml"));
        println!(
            "\n{}",
            record2.to_xml_formatted().expect("We made some xml")
        );
    }

    if bin_file_op.is_some() {

        let mut iter = Record::from_binary_file(&bin_file_op.unwrap()).expect("Parse Binary File");

        let record = iter.next().unwrap();

        //println!("bin record: {:?}", record);
        println!("Binary record as breaker:\n{}", record.to_breaker());
        println!("\nBinary record as xml:\n{}", record.to_xml_formatted().unwrap());
    }
}
