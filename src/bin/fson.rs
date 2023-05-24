use fuzzy_json::fson;

extern crate structopt;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(name = "INPUT", default_value = "-")]
    input_cumin: String,
}

fn cat(file_name: &String) -> String {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::{self, Read};

    let mut content = String::new();
    if file_name == "-" {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut content).unwrap();
    } else {
        let file = File::open(&file_name).unwrap();
        let mut buf_reader = BufReader::new(file);
        buf_reader.read_to_string(&mut content).unwrap();
    }
    content
}

fn main() {
    let opt = Opt::from_args();
    let content = cat(&opt.input_cumin);
    if let Some(data) = fson(content.as_str()) {
        println!("{}", data);
    } else {
        std::process::exit(1);
    }
}
