use std::env;
use getopts;

pub mod marc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("f", "file", "MARC XML File", "MARCXML_FILE");

    let params = opts.parse(&args[1..]).unwrap();

    if let Some(filename) = params.opt_str("file") {

        //println!("Reading XML file: {}", filename);

        let record =
            marc::Record::from_xml_file(&filename).expect("MARCXML File Parse");

        //println!("Record\n{}", record);

        //println!("{}", record.to_xml().expect("MARC to XML OK"));

        let breaker = record.to_breaker();
        let record2 = marc::Record::from_breaker(&breaker).expect("Create record from breaker");
        println!("{}", record2.to_breaker());
        println!("{}", record2.to_xml().expect("We made some xml"));
    }
}



