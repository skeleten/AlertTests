#[macro_use] extern crate nom;
extern crate chrono;
extern crate rss;
extern crate hyper;

use ::nom::{IResult, digit, eof};
use std::io::{Write, Read};

#[derive(Debug)]
pub struct Alert {
    pub rewards: Vec<String>,
    pub time: Option<usize>,
    pub location: String,
    pub description: Option<String>,
}

impl Alert {
    pub fn new(loc: String) -> Self {
        Alert {
            location: loc,
            rewards: Vec::new(),
            time: None,
            description: None,
        }
    }

    pub fn parse(inp: &str) -> Option<Self> {
        let strs: Vec<&str> = inp.split('-').map(|s| s.trim()).collect();
        let len = strs.len();
        // does it have a time
        let time_result = time(strs[len-1].as_bytes());
        let pos_loc;
        let mut rewards = Vec::new();

        if !time_result.is_done() {
            pos_loc = len - 1;
        } else {
            pos_loc = len - 2;
        };

        let time = match time_result {
            IResult::Done(_,o) => Some(o),
            _ => None,
        };
        for i in 0..pos_loc {
            let s = strs[i];
            rewards.push(s.to_string());
        }
        let loc = strs[pos_loc];

        Some(Alert {
            location: loc.to_string(),
            rewards: rewards,
            time: time,
            description: None,
        })
    }
}

named!(rewards<Vec<String> >,
       many1!(complete!(chain!(r:  take_until!(" - ")~
                               tag!(" - "),
                               || String::from_utf8_lossy(r).into_owned()))));
named!(location<String>,
       chain!(s: take_until!(" - ") ~
              tag!(" - "),
              || String::from_utf8_lossy(s).into_owned()));

named!(time<usize>,
       map_res!(chain!(d: take_until!("m") ~
                       tag!("m"),
                       || d),
                |s| {
                    let s = String::from_utf8_lossy(s);
                    s.parse::<usize>()
                }));

pub fn get_feed() -> Option<String> {
    let client = hyper::Client::new();
    let mut res = match client.get("http://content.warframe.com/dynamic/rss.php").send() {
        Ok(r) => r,
        Err(_) => return None,
    };
    if res.status != hyper::Ok {
        None
    } else {
        let mut s = String::new();
        match res.read_to_string(&mut s) {
            Ok(_) => { },
            Err(_) => return None,
        };
        Some(s)
    }
}

fn main() {
    let feed = get_feed().unwrap();
    let r: rss::Rss = feed.parse().unwrap();
    for item in r.0.items {
        let alert = Alert::parse(&item.title.unwrap_or("".to_string())).unwrap();
        let time = chrono::DateTime::parse_from_rfc2822(&item.pub_date.unwrap_or("".to_string())).unwrap();
        println!("{:?}: {:#?}", time, alert);
    }
}
