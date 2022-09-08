use std::env;
use getopts;
use marcutil::Record;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("f", "file", "MARC XML File", "MARCXML_FILE");

    let params = opts.parse(&args[1..]).expect("Options parsed");

    if let Some(filename) = params.opt_str("file") {

        let record =
            Record::from_xml_file(&filename).expect("MARCXML File Parse");

        println!("{}", record.to_xml().expect("MARC to XML OK"));

        let breaker = record.to_breaker();

        let record2 = Record::from_breaker(&breaker).expect("Create record from breaker");

        println!("\n{}", record2.to_breaker());
        println!("\n{}", record2.to_xml().expect("We made some xml"));
    }
}



