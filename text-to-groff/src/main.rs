use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;
use std::str;
use regex::RegexSet;


#[derive(Debug)]
struct Pair(String,String);

impl str::FromStr for Pair {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) ->  Result<Self,Self::Err>{
        let vec = s.trim_start_matches("[").trim_end_matches("]").split(",").collect::<Vec<&str>>();
        Ok(Pair(vec[0].to_string(),vec[1].to_string()))
    }
}

struct Rules {
    set:RegexSet,
    values:Vec<String>
}

impl Rules {
    fn get(&self,s: &str) -> Option<String>{
        let matches = self.set.matches(s);
        if matches.matched_any() {
            Some(self.values[matches.into_iter().collect::<Vec<_>>()[0]].to_owned())
        } else {
            None
        }
    }
}

impl FromIterator<Pair> for Rules {
    fn from_iter<I: IntoIterator<Item=Pair>>(iter: I) -> Self {
        let mut set:Vec<String> = Vec::new();
        let mut values:Vec<String> = Vec::new();

        for i in iter {
            set.push(i.0);
            values.push(i.1);
        }

        Rules {
            set: RegexSet::new(set).expect("Error whilst compiling rules"),
            values:values
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
    input: PathBuf,
    /// The path of the desired output.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// A vector of rules
    #[structopt(short, long)]
    rules: Vec<Pair>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::from_args();
    let mut buf:Vec<String> = Vec::new();
    let input = fs::read_to_string(args.input)?;
    let rules: Rules = args.rules.into_iter().collect();
    
    


    for word in input.split(" ") {
        match rules.get(word) {
            Some(rule)=>{
                println!("{}",rule);
            },
            _ => {
                buf.push(word.to_string())
            }
        }
    };


    Ok(())
}