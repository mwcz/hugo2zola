#![feature(stdin_forwarders)]

// use std::convert::From;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::io::{self};
use toml;

enum FMScanState {
    Waiting,
    Started,
    // Complete,
}

#[derive(Serialize, Deserialize, Debug)]
struct FrontMatter {
    title: String,
    date: String,
    description: String,
    category: String,
    tags: Vec<String>,
    draft: bool,
    slug: String,
    aliases: Vec<String>,
}

fn main() {
    let lines = io::stdin().lines();

    let mut fm_serialized: Vec<String> = Vec::new();
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

        fm_serialized.push(line);
    }

    for fm_line in fm_serialized {
        println!("{}", fm_line);
    }

    let fm = FrontMatter {
        title: "Cool Blog Post".into(),
        date: "2022-01-10".into(),
        description: "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.".into(),
        draft: false,
        slug: "cool-blog-post".into(),
        aliases: vec!["/old-cool-blog-post-url/".into()],
        category: "Category".into(),
        tags: vec!["tag1".into(),"tag2".into(), "tag3".into()],
    };

    println!("{}", toml::to_string(&fm).unwrap());
    println!("{}", serde_yaml::to_string(&fm).unwrap());
}
