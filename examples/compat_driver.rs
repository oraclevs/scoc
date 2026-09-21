use std::env;
use std::fs;

use scoc::{OptionValue, ParseOptions};

fn main() {
    let mut parser = None;
    let mut fixture = None;
    let mut raw = false;
    let mut streaming = false;
    let mut ignore_errors = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--parser" => parser = args.next(),
            "--fixture" => fixture = args.next(),
            "--raw" => raw = true,
            "--streaming" => streaming = true,
            "--ignore-errors" => ignore_errors = true,
            other => panic!("unknown argument: {other}"),
        }
    }
    let parser = parser.expect("--parser required");
    let fixture = fixture.expect("--fixture required");
    let bytes = fs::read(fixture).expect("read fixture");
    let mut options = ParseOptions::default();
    if raw {
        options.insert("raw", OptionValue::Bool(true));
    }
    if ignore_errors {
        options.insert("ignoreErrors", OptionValue::Bool(true));
    }

    let value = if streaming {
        let mut stream = scoc::stream_parser(&parser, &options).expect("create stream parser");
        let mut rows = stream.push(&bytes).expect("parse stream bytes");
        rows.extend(stream.finish().expect("finish stream parser"));
        serde_json::Value::Array(rows)
    } else {
        scoc::parse(&parser, &bytes, &options).expect("parse fixture")
    };
    println!(
        "{}",
        serde_json::to_string(&value).expect("serialize comparison output")
    );
}
