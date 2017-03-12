extern crate clap;
extern crate toml;

use std::cmp::Ordering;
use std::io;
use std::io::{Read, Write};
use std::error;
use std::fmt;
use std::fs::{File, OpenOptions};

use clap::{Arg, App};

#[derive(Debug)]
enum CliError {
    UnexpectedValue,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CliError::UnexpectedValue => write!(f, "Unexpected Value Error"),
        }
    }
}

impl error::Error for CliError {
    fn description(&self) -> &str {
        match self {
            &CliError::UnexpectedValue => "Unexpected value given",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            _ => None,
        }
    }
}

fn main() {
    let matches = App::new("tomlsort")
        .arg(Arg::with_name("target")
            .short("t")
            .long("target-array")
            .value_name("ARRAY")
            .required(true)
            .help("Sets a sort target array")
            .takes_value(true))
        .arg(Arg::with_name("key")
            .short("k")
            .long("sort-key")
            .value_name("KEY")
            .required(true)
            .help("Sets a key to use for sorting, if the target is an array of objects"))
        .arg(Arg::with_name("out")
            .short("o")
            .long("output")
            .value_name("OUTFILE")
            .help("Sets a output file"))
        .arg(Arg::with_name("path")
            .value_name("PATH")
            .help("Sets PATH of a target toml file")
            .required(true))
        .get_matches();
    let target = matches.value_of("target").unwrap();
    let key = matches.value_of("key").unwrap();
    let path = matches.value_of("path").unwrap();
    let mut out: Box<io::Write> = matches.value_of("out")
        .and_then(|p| OpenOptions::new().write(true).create(true).open(p).ok())
        .map(|f| Box::new(f) as Box<io::Write>)
        .unwrap_or(Box::new(io::stdout()) as Box<io::Write>);

    let mut s = String::new();
    let mut file = match File::open(&path) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to open file '{}': {}", path, e);
            ::std::process::exit(1);
        }
    };
    let _ = file.read_to_string(&mut s);
    let mut toml = match s.parse::<toml::Value>() {
        Ok(v) => v,
        Err(e) => {
            println!("failed to parse toml file: {}", e);
            ::std::process::exit(1);
        }
    };
    match tomlsort(&mut toml, target, key) {
        Ok(_) => {}
        Err(e) => {
            println!("failed to sort toml: {}", e);
            ::std::process::exit(1);
        }
    };
    let _ = write!(out, "{}", toml);
}

fn tomlsort<S: ?Sized>(v: &mut toml::Value, k: &S, k2: &S) -> Result<(), CliError>
    where S: AsRef<str>
{
    let mut arr =
        v.get_mut(&k.as_ref()).and_then(|t| t.as_array_mut()).ok_or(CliError::UnexpectedValue)?;
    arr.sort_by(|a, b| match (a, b) {
        (ta @ &toml::Value::Table(_), tb @ &toml::Value::Table(_)) => {
            let sa = ta.get(k2.as_ref()).and_then(|s| s.as_str()).unwrap_or_default();
            let sb = tb.get(k2.as_ref()).and_then(|s| s.as_str()).unwrap_or_default();
            sa.cmp(&sb)
        }
        _ => Ordering::Equal,
    });
    Ok(())
}

#[cfg(test)]
mod test {
    extern crate toml;
    use toml::Value;
    use super::tomlsort;

    #[test]
    fn test_sort_string() {
        let mut expected = vec!["foo", "bar", "baz"];
        expected.sort();
        let mut toml_array = r#"[[arr]]
        name = "foo"
        [[arr]]
        name = "bar"
        [[arr]]
        name = "baz""#
            .parse::<Value>()
            .unwrap();
        let _ = tomlsort(&mut toml_array, "arr", "name").unwrap();
        let res: Vec<String> = toml_array.get("arr")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .flat_map(|t| t.get("name"))
            .flat_map(|s| s.as_str())
            .map(|s| String::from(s))
            .collect();
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);
    }

}
