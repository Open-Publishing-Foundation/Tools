use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;
use std::str;
use std::io::{self};
use std::io::prelude::*;
use regex::RegexSet;
use itertools::Itertools;

trait NextN {
    fn next_n(&mut self,n:usize) -> Vec<&str>;
}

impl NextN for str::Split<'_, &str> {
    fn next_n(&mut self,n:usize) -> Vec<&str> {
        let i = if n < self.count() {n} else {self.count()};
        let mut v = Vec::with_capacity(i);
        for _ in 0..i {
            match self.next(){
                Some(e) => v.push(e),
                None => unreachable!()
            }
        }
        v
    }
}

#[derive(Debug)]
struct Pair(String,String);

impl str::FromStr for Pair {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) ->  Result<Self,Self::Err>{
        let vec = s.trim_start_matches('[').trim_end_matches(']').split(',').collect::<Vec<&str>>();
        Ok(Pair(vec[0].to_string(),vec[1].to_string()))
    }
}


struct Rules {
    set:RegexSet,
    values:Vec<String>
}

impl Rules {
    fn get(&self,s: &str) -> Option<&str>{
        match self.set.matches(s).into_iter().next() {
            Some(e) => Some(&self.values[e]),
            _ => None
        }
    }
}

impl FromIterator<Pair> for Rules {
    fn from_iter<I: IntoIterator<Item=Pair>>(iter: I) -> Self {
        let (set, values): (Vec<_>, Vec<_>) = iter.into_iter().map(|Pair(a,b)| (a, b)).unzip();
        Rules {
            set: RegexSet::new(set).expect("Error whilst compiling rules"),
            values
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
    #[structopt(short, long)]
    rules: Vec<Pair>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::from_args();
    let rules: Rules = args.rules.into_iter().collect();
    let input = match args.input {
        Some(e) => fs::read_to_string(e)?,
        _ =>  {
            let mut buf = String::new();
            io::stdin().lock().read_to_string(&mut buf)?;
            buf
        }
    };

    let mut input = input.split(" ");
    let mut buf: Vec<String> = Vec::new();

    while let Some(word) = input.next() {
        buf.push(
            match rules.get(word) {
                Some(rule) =>{
                    // Interleave desired words into macro
                    rule.split("{}").interleave(
                        input.next_n(rule.matches("{}").count())
                    ).collect()
                },
                _ => word.to_string()
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