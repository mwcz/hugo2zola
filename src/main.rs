#![feature(stdin_forwarders)]

use regex::Regex;
use serde;
use serde_yaml;
use std::convert::From;
use std::io;
use toml;

enum FMScanState {
    Waiting,
    Started,
    // Complete,
}

/// Generic wrapper that allow one or more occurrences of specified type.
///
/// In YAML it will presented or as a value, or as an array:
/// ```yaml
/// one: just a string
/// many:
///   - 1st string
///   - 2nd string
/// ```
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// Single value
    One(T),
    /// Array of values
    Vec(Vec<T>),
}
impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(from: OneOrMany<T>) -> Self {
        match from {
            OneOrMany::One(val) => vec![val],
            OneOrMany::Vec(vec) => vec,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct HugoFM {
    title: String,
    date: String,
    description: String,
    category: String,
    tags: Vec<String>,
    draft: bool,
    slug: String,
    aliases: OneOrMany<String>,
}

fn main() {
    let lines = io::stdin().lines();

    let mut fm_se: Vec<String> = Vec::new();
    let mut fm_scan_state = FMScanState::Waiting;
    let yaml_tag_re = Regex::new(r"---\s*").unwrap();

    for line_r in lines {
        let line = line_r.unwrap();
        if yaml_tag_re.is_match(&line) {
            match fm_scan_state {
                FMScanState::Waiting => {
                    fm_scan_state = FMScanState::Started;
                    continue; // front matter begun, skip to the next line before deserializing
                }
                FMScanState::Started => {
                    // fm_scan_state = FMScanState::Complete;
                    break;
                }
            };
        }

        fm_se.push(line);
    }

    for fm_line in &fm_se {
        println!("{}", fm_line);
    }

    let fm_str = fm_se.join("\n");

    let fm_de: HugoFM = serde_yaml::from_str(&fm_str).unwrap();

    println!("{:?}", fm_de);

    // let fm = HugoFM {
    //     title: "Cool Blog Post".into(),
    //     date: "2022-01-10".into(),
    //     description: "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.".into(),
    //     draft: false,
    //     slug: "cool-blog-post".into(),
    //     aliases: vec!["/old-cool-blog-post-url/".into()],
    //     category: "Category".into(),
    //     tags: vec!["tag1".into(),"tag2".into(), "tag3".into()],
    // };

    // println!("{}", toml::to_string(&fm).unwrap());
    // println!("{}", serde_yaml::to_string(&fm).unwrap());
}
