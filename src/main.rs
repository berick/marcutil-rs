use std::env;
use getopts;

pub mod marc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("f", "file", "MARC XML File", "MARCXML_FILE");

    let params = opts.parse(&args[1..]).unwrap();

    if let Some(filename) = params.opt_str("file") {
        println!("Reading XML file: {}", filename);
        let record =
            marc::Record::from_xml_file(&filename).expect("MARCXML File Parse");
        println!("Record\n{}", record);
    }
}


