#![feature(stdin_forwarders)]

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, OneOrMany};
use serde_yaml;
use std::convert::From;
use std::io::{self, Lines};
use std::{collections::HashMap, io::StdinLock};
use tera::{Map, Value};
use toml;

enum FMScanState {
    Waiting,
    Started,
    // Complete,
}

/// Parse a string for a datetime coming from one of the supported TOML format
/// There are three alternatives:
/// 1. an offset datetime (plain RFC3339)
/// 2. a local datetime (RFC3339 with timezone omitted)
/// 3. a local date (YYYY-MM-DD).
/// This tries each in order.
fn parse_datetime(d: &str) -> Option<NaiveDateTime> {
    DateTime::parse_from_rfc3339(d)
        .or_else(|_| DateTime::parse_from_rfc3339(format!("{}Z", d).as_ref()))
        .map(|s| s.naive_local())
        .or_else(|_| NaiveDate::parse_from_str(d, "%Y-%m-%d").map(|s| s.and_hms(0, 0, 0)))
        .ok()
}

/// Used as an attribute when we want to convert from TOML to a string date
/// If a TOML datetime isn't present, it will accept a string and push it through
/// TOML's date time parser to ensure only valid dates are accepted.
/// Inspired by this proposal: https://github.com/alexcrichton/toml-rs/issues/269
fn from_toml_datetime<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use std::str::FromStr;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeDatetime {
        Datetime(toml::value::Datetime),
        String(String),
    }

    match MaybeDatetime::deserialize(deserializer)? {
        MaybeDatetime::Datetime(d) => Ok(Some(d.to_string())),
        MaybeDatetime::String(s) => match toml::value::Datetime::from_str(&s) {
            Ok(d) => Ok(Some(d.to_string())),
            Err(e) => Err(D::Error::custom(e)),
        },
    }
}

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
/// See: https://gohugo.io/content-management/front-matter/#front-matter-variables
struct HugoFM {
    title: Option<String>,
    date: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    #[serde(default)]
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    draft: Option<bool>,
    slug: Option<String>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    #[serde(default)]
    aliases: Option<Vec<String>>,

    // my custom front matter additions which are destined for the [extras] section in zola fm
    lastmod: Option<String>,
    snapdate: Option<String>,
    photo_id: Option<String>,
    palette0: Option<String>,
    palette1: Option<String>,
    image: Option<String>,
    thumbnail: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde_as]
struct ZolaFM {
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    date: Option<String>,
    draft: Option<bool>,
    slug: Option<String>,
    path: Option<String>,
    aliases: Option<Vec<String>>,
    taxonomies: HashMap<String, Vec<String>>,
    extra: Map<String, Value>,
}

impl From<HugoFM> for ZolaFM {
    fn from(hugo: HugoFM) -> Self {
        // create the Zola front matter [extra] section, made up of my custom Hugo front matter
        // properties

        let mut extra = Map::new();

        if let Some(snapdate) = hugo.snapdate {
            extra.insert("snapdate".into(), Value::String(snapdate));
        };

        if let Some(photo_id) = hugo.photo_id {
            extra.insert("photo_id".into(), Value::String(photo_id));
        };

        if let Some(palette0) = hugo.palette0 {
            extra.insert("palette0".into(), Value::String(palette0));
        };

        if let Some(palette1) = hugo.palette1 {
            extra.insert("palette1".into(), Value::String(palette1));
        };

        if let Some(image) = hugo.image {
            extra.insert("image".into(), Value::String(image));
        };

        if let Some(thumbnail) = hugo.thumbnail {
            extra.insert("thumbnail".into(), Value::String(thumbnail));
        };

        Self {
            title: hugo.title,
            description: hugo.description,
            updated: hugo.lastmod.or(hugo.date.clone()),
            date: hugo.date,
            draft: hugo.draft,
            slug: hugo.slug,
            path: hugo.alias,
            aliases: hugo.aliases,
            extra,
            taxonomies: HashMap::new(), // TODO convert categories & tags to zola taxonomies
        }
    }
}

fn read_hugo_fm(text: Lines<StdinLock>) -> HugoFM {
    let mut fm_se: Vec<String> = Vec::new();
    let mut fm_scan_state = FMScanState::Waiting;
    let yaml_tag_re = Regex::new(r"---\s*").unwrap();
    let yaml_field_re = Regex::new(r"(?P<field>.*?)\s*:").unwrap();

    for line_r in text {
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

        let yaml_field_name = yaml_field_re.find(&line);

        let line = match yaml_field_name {
            Some(name) => yaml_field_re
                .replace(&line, name.as_str().to_lowercase())
                .into(),
            None => line,
        };

        fm_se.push(line);
    }

    let hugo_fm_str = fm_se.join("\n"); // join into a single string

    let hugo_fm: HugoFM = serde_yaml::from_str(&hugo_fm_str).unwrap(); // decode hugo fm's yaml

    hugo_fm
}

fn main() {
    // read hugo blog post from stdin, extract and decode front matter
    let lines = io::stdin().lines();
    let hugo_fm = read_hugo_fm(lines);

    // convert hugo fm into zola fm
    let zola_fm = ZolaFM::from(hugo_fm);

    // pretty-print it
    print!("+++\n{}+++", toml::to_string_pretty(&zola_fm).unwrap());
}
