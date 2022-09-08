use getopts;
use marcutil::Record;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("x", "xml-file", "MARC XML File", "MARCXML_FILE");

    let params = opts.parse(&args[1..]).expect("Options parsed");

    let file_op = params.opt_str("xml-file");

    if file_op.is_none() {
        eprintln!("MARC XML file required");
        return;
    }

    let filename = file_op.unwrap();

    let record = Record::from_xml_file(&filename).expect("MARCXML File Parse");

    if let Some(title) = record.get_values("245", "a").first() {
        println!("Maintitle => {title}");
    }

    println!("{}", record.to_xml().expect("MARC to XML OK"));

    let breaker = record.to_breaker();

    let record2 = Record::from_breaker(&breaker).expect("Create record from breaker");

    println!("\n{}", record2.to_breaker());
    println!("\n{}", record2.to_xml().expect("We made some xml"));
}
