#[macro_use]
extern crate lazy_static; // ensure that regular expressions are compiled exactly once

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate regex;
extern crate walkdir;
extern crate cld2;
extern crate encoding;
extern crate time;
extern crate uuid;
extern crate core;
extern crate users;
extern crate curl;
extern crate rand;
extern crate ammonia;
extern crate sha1;
extern crate flame;
extern crate term;
extern crate difference;
extern crate flate2;
extern crate bincode;
extern crate rustc_serialize;

// extern crate rsgenetic;
// #[macro_use] extern crate nickel;
// use nickel::Nickel;
use std::os::unix::fs::MetadataExt; /* Metadata trait */
use uuid::Uuid;
use regex::Regex;
use core::result::Result;
use time::{get_time, precise_time_ns};
use cld2::{detect_language, Format, Reliable, Lang};
use walkdir::{WalkDir}; // DirEntry, WalkDirIterator
use users::get_user_by_uid;
use std::io::prelude::{Read,Write};
use std::io::{BufReader, BufWriter};
use std::fs::{File, OpenOptions};
use std::path::Path;
use curl::http;
use rustc_serialize::json;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::ZlibEncoder;
use bincode::rustc_serialize::{encode, decode};

// local
mod structs;
mod utils;
mod base;
mod process;

use utils::*;
use structs::*;
use base::*;
use process::*;


fn main() {
    env_logger::init().unwrap();

    let start = precise_time_ns();
    let mut files_processed = 0;
    let mut files_skipped = 0;

    for user in fetch_users() {
        let path = format!("/home/{}/", user.name());
        if ! Path::new(path.as_str()).exists() {
            debug!("Path doesn't exists: {}. Skipping", path);
            continue
        }

        let mut changeset = Changeset {
            uuid: Uuid::new_v4(),
            parent: root_uuid(), // XXX - should be attached to "root branch"
            timestamp: time::precise_time_ns() / 1000 / 1000,
            entries: Vec::new(),
        };

        info!("Traversing path: '{}'", path);
        let walker = WalkDir::new(path)
            .follow_links(false)
            .max_depth(4)
            .max_open(512)
            .into_iter();

        for entry in walker /* filter everything we don't have access to */
                        .filter_map(|e| e.ok())
                        .filter(|e| e.metadata().unwrap().is_file() && e.path().to_str().unwrap_or("").contains("domains")) {

            let entry_name = format!("path: {}", entry.path().to_str().unwrap_or("NO-FILE"));
            flame::start(entry_name.clone());

            match process_domain(entry.path()) {
                Some(domain_entry) => {
                    /* write flamegraph */
                    flame::end(entry_name.clone());
                    let graph_file_name = format!("{}-{}.svg", user.name(), domain_entry.name);
                    match flame::dump_svg(&mut File::create(graph_file_name).unwrap()) {
                        Ok(_) => debug!("Graph stored successfully"),
                        Err(err) => warn!("Failed to store graph: {}", err),
                    }
                    flame::clear();

                    changeset.entries.push(domain_entry);
                    files_processed += 1;
                },
                None => {
                    files_skipped += 1;
                },
            }
        }

        /* write changeset serialized to json */
        let output_file = format!("{}_{}.chgset.json", user.name(), changeset.uuid);
        match OpenOptions::new()
                            .read(true)
                            .create(true)
                            .write(true)
                            .append(false)
                            .open(output_file.clone()) {
            Ok(f) => {
                let mut writer = BufWriter::new(f);
                match writer.write(changeset.to_string().as_bytes()) {
                    Ok(some) => {
                        debug!("Changeset: {} has been stored in: {} ({} bytes)", changeset.uuid, output_file, some)
                    },
                    Err(err) => {
                        error!("Error: {}, file: {}", err, output_file)
                    }
                }
            },
            Err(err) => {
                error!("File open error: {}, file: {}", err, output_file)
            }
        }

        /* now write compressed binary changeset */
        let changeset_dir = format!(".changesets/{}", user.name());
        match std::fs::create_dir_all(changeset_dir.clone()) {
            Ok(_) => {},
            Err(err) => error!("{:?}", err),
        }
        let file_name = format!("{}/{}-{}.chgset", changeset_dir, changeset.uuid, changeset.timestamp);
        let binary_encoded = encode(&changeset, bincode::SizeLimit::Infinite).unwrap();

        let mut zlib = ZlibEncoder::new(Vec::new(), Compression::Best);
        zlib.write(&binary_encoded[..]).unwrap();
        let compressed_bytes = zlib.finish().unwrap();

        let mut writer = BufWriter::new(File::create(file_name.clone()).unwrap());
        let bytes_written = writer.write(&compressed_bytes).unwrap();
        info!("Changeset stored: {} ({} bytes)", file_name, bytes_written);
    }

    let end = precise_time_ns();
    info!("Traverse for: {} files, (skipped: {} files), elapsed: {} miliseconds", files_processed, files_skipped, (end - start) / 1000 / 1000);

    // let mut server = Nickel::new();
    // server.utilize(router! {
    //     get "**" => |_req, _res| {
    //         "Hello world!"
    //     }
    // });
    // server.listen("127.0.0.1:6000");
}


#[cfg(test)]
#[test]
fn fetch_users_test() {
    for user in fetch_users() {
        if user.name() == "root" ||
           user.name() == "toor" {
            assert!(user.uid() == 0);
        } else {
            assert!(user.uid() > 0);
            assert!(user.name().len() > 0);
        }
    }
}


#[cfg(test)]
#[test]
fn language_detection_test() {
    let _ = env_logger::init();
    let texts = vec!("Młody Amadeusz szedł suchą szosą.", "Mladý Amadeusz išiel suchej ceste.", "Young Amadeus went a dry road.");
    let expected = vec!("pl", "sk", "en");
    for (text, res) in texts.iter().zip(expected.iter()) {
        match detect_language(text, Format::Text).0 { /* ignore detection reliability here */
            Some(Lang(lang)) => assert!(lang.to_string() == res.to_string()),
            _ => assert!(1 == 0, "Language not recognized properly!"),
        }
    }
}


#[cfg(test)]
#[test]
fn regex_group_domain_extractor_test() {
    let _ = env_logger::init();
    let domain_correct = vec!(
        "/home/user/domains/domain.tld.my.pl/public_html/file.php",
        "/home/user2/domains/domena.pl/public_html/index.html",
    );
    let correct_results = vec!(
        "domain.tld.my.pl",
        "domena.pl"
    );
    for (file, res) in domain_correct.iter().zip(correct_results.iter()) {
        let entry = structs::FileEntry {
            owner: Owner {
                origin: String::new(), /* XXX */
                name: String::from("root"),
                account_type: structs::AccountType::Admin,
                .. Default::default()
            },
            path: file.to_string(),
            size: 123,
            mode: 0711 as u32,
            modified: 1,
            encoding: "UTF-8".to_string(),
            .. Default::default()
        };

        let domain_from_path = Regex::new(r".*/domains/(.*)/public_html/.*").unwrap();
        for _domain in domain_from_path.captures_iter(entry.path.as_str()) {
            debug!("Domain: {:?}", _domain.at(1).unwrap());
            assert!(_domain.at(1).unwrap() == res.to_string());
        }
    }
}
