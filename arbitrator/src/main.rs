use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;
use std::str;
use std::io::{self};
use std::io::prelude::*;
use regex::RegexSet;
use itertools::Itertools;
use serde_json::{Map,Value};

/// Get the next n items from an iterator
trait NextN {
    /// Get the next n items from an iterator
    fn next_n(&mut self,n:usize) -> Vec<&str>;
}

impl NextN for str::Lines<'_> {
    fn next_n(&mut self,n:usize) -> Vec<&str> {
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(e) = self.next(){v.push(e)}
        }
        v
    }
}

#[derive(Debug)]
struct Rules {
    set:RegexSet,
    values:Vec<String>,
    counts:Vec<usize>
}

impl Rules{
    fn get(&self,s: &str) -> Option<(&str,usize)>{
        self.set
            .matches(s)
            .into_iter()
            .next()
            .map(|e| (self.values[e].as_str(),self.counts[e]))
    }
}

impl FromIterator<(String, Value)> for Rules {
    fn from_iter<I: IntoIterator<Item=(String, Value)>>(iter: I) -> Self {
        let (set, values): (Vec<_>, Vec<_>) = iter.into_iter()
            .map(|(a,b)| (a, b.as_str().expect("Error converting Json into rules.").to_string()))
            .unzip();
        
        let mut counts = Vec::with_capacity(values.len());
        for v in &values {
            counts.push(v.matches("{}").count())
        }
        
        Rules {
            set: RegexSet::new(set).expect("Error whilst compiling rules"),
            values,
            counts
        }
    }
}

/// Arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "text-to-groff",
    about = "Convert text to groff/troff format.",
    author = "Oliver Brotchie, o.brotchie@gmail.com"
)]
struct Args {
    /// The path to your desired input.
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,
    /// The path of the desired output.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// A vector of rules
    /// The path/s to one or more JSON files containing the variables.
    #[structopt(short, long, parse(from_os_str))]
    rules: Vec<PathBuf>,
}

#[allow(clippy::single_char_pattern)]
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::from_args();
    let rules: Rules = merge_json(args.rules)?;
    let input = match args.input {
        Some(e) => fs::read_to_string(e)?,
        _ =>  {
            let mut buf = String::new();
            io::stdin().lock().read_to_string(&mut buf)?;
            buf
        }
    };

    let mut input = input.lines();
    let mut buf: Vec<String> = Vec::new();

    // Iterate over each line and apply rules where needed.
    while let Some(line) = input.next() {
        buf.push(
            match rules.get(line) {
                Some((rule,count)) =>{

                    // Test if the rule has any inserts
                    if count == 0 {
                        if rule.is_empty() {continue} 
                        else {rule.to_string()}
                    } else {
                        
                        // If it does, interpollate the requiered number of lines.
                        let mut v = Vec::with_capacity(count);
                        v.push(line);
                        for _ in 0..count-1 {
                            if let Some(e) = input.next(){v.push(e)}
                        }
                        rule.split("{}").interleave(v).collect()
                    }
                },
                None => line.to_string()
            }
        )
    }

    // Output
    match args.output {
        Some(path) => fs::write(path,buf.join("\n"))?,
        None => buf.into_iter().for_each(|e| println!("{}",e))
    }

    Ok(())
}

/// Read in and merge all specified Json files.
fn merge_json(paths:Vec<PathBuf>)->Result<Rules, Box<dyn std::error::Error>> {

    let mut json = Map::new();
    for p in paths {
        json.extend(
            match serde_json::from_str::<Value>(&fs::read_to_string(p)
                .expect("Error: Json file was not found, please specify a valid path."))
            ?.as_object() {
                Some(object) => object.to_owned(),
                None => continue
            }
        );
    }

    Ok(json.into_iter().collect())
}