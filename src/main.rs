#[macro_use] extern crate lazy_static; // ensure that regular expressions are compiled exactly once
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;

// #[macro_use] extern crate nickel;
// use nickel::Nickel;
extern crate walkdir;
extern crate cld2;
extern crate encoding;
extern crate ammonia;
extern crate time;
extern crate sha1;
extern crate uuid;
extern crate core;
extern crate rustc_serialize;
extern crate users;
extern crate curl;
extern crate rsgenetic;
extern crate rand;

mod structs;

use uuid::Uuid;
use regex::Regex;
use core::result::Result;
use time::*;
use ammonia::*;
use cld2::{detect_language, Format, Reliable, Lang};
use encoding::types::*;
use encoding::all::*;
use walkdir::{WalkDir}; // DirEntry, WalkDirIterator
use users::get_user_by_uid;
use rustc_serialize::json; // Encodable, Decodable
use std::env;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use std::collections::{HashSet};
use std::os::unix::fs::MetadataExt;
use curl::http;

// local
use structs::*;


fn detect_encoding(vec: &Vec<u8>) -> Option<EncodingRef> {
    let possible_encodings = vec!(
        ASCII as EncodingRef,
        WINDOWS_1250 as EncodingRef,
        UTF_8 as EncodingRef,
        UTF_16LE as EncodingRef,
        UTF_16BE as EncodingRef,
        ISO_8859_1 as EncodingRef,
        ISO_8859_2 as EncodingRef,
        ISO_8859_3 as EncodingRef,
        ISO_8859_4 as EncodingRef,
        ISO_8859_5 as EncodingRef,
        ISO_8859_6 as EncodingRef,
        ISO_8859_7 as EncodingRef,
        ISO_8859_8 as EncodingRef,
        ISO_8859_10 as EncodingRef,
        ISO_8859_13 as EncodingRef,
        ISO_8859_14 as EncodingRef,
        ISO_8859_15 as EncodingRef,
        ISO_8859_16 as EncodingRef,
        KOI8_R as EncodingRef,
        KOI8_U as EncodingRef,
        MAC_ROMAN as EncodingRef,
        WINDOWS_874 as EncodingRef,
        WINDOWS_949 as EncodingRef,
        WINDOWS_1251 as EncodingRef,
        WINDOWS_1252 as EncodingRef,
        WINDOWS_1253 as EncodingRef,
        WINDOWS_1254 as EncodingRef,
        WINDOWS_1255 as EncodingRef,
        WINDOWS_1256 as EncodingRef,
        WINDOWS_1257 as EncodingRef,
        WINDOWS_1258 as EncodingRef,
    );
    let in_trap = DecoderTrap::Strict;

    for encoding in possible_encodings {
        match encoding.decode(&vec, in_trap) {
            Ok(_) => return Some(encoding),
            Err(_) => Some(ERROR),
        };
    }
    None
}


fn read_fragment<R>(reader: R, bytes_to_read: u64) -> Option<Vec<u8>> where R: Read {
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    match chunk.read_to_end(&mut buf) {
        Ok(_) =>
            Some(buf),
        _ =>
            None,
    }
}


fn valid_file_extensions(name: &str) -> bool {
    lazy_static! {
        /*
        Regex will be compiled when it's used for the first time
        On subsequent uses, it will reuse the previous compilation
        */
        static ref RE: Regex = Regex::new(r"\.(php[0-9]*|[s]?htm[l0-9]*|txt|inc|py|pl|rb|sh|[xyua]ml|htaccess|rss|[s]?css|js|mo|po|ini|ps|l?a?tex)$").unwrap();
    }
    RE.is_match(name)
}


#[test]
fn matcher_test() {
    for valid in vec!(
        "somestrange123.file.php", ".htm", "a.txt", "file.html", "file.htm4",
        "exym.pl", "404.shtml", "album.rss", "a.ps", "a.latex", "mr.tex"
    ) {
        assert!(valid_file_extensions(valid));
    }
    for invalid in vec!("file.plo", "file.pyc", ".phpa", "somefile", "file.pshtml") {
        assert!(!valid_file_extensions(invalid));
    }
}


fn sha1_of(input: String) -> String {
    let mut m = sha1::Sha1::new();
    m.update(input.as_bytes());
    m.hexdigest()
}


/* html tag cleaner PoC: */
fn strip_html_tags(binary_content: &Vec<u8>) -> String {
    let a_buf = String::from_utf8_lossy(&binary_content);
    lazy_static! {
        static ref TAGS: Ammonia<'static> = Ammonia{
            tags: HashSet::new(), /* list of tags that may stay in content - strip all */
            .. Ammonia::default()
        };
    }
    let cleaned = String::from(TAGS.clean(&a_buf));
    let matches: &[_] = &['\n', '\t', '\r'];
    cleaned.replace(matches, "").to_string()
}

fn strip_html_tags_slice(binary_content: &[u8]) -> String {
    let a_buf = String::from_utf8_lossy(binary_content);
    lazy_static! {
        static ref TAGS2: Ammonia<'static> = Ammonia {
            tags: HashSet::new(), /* list of tags that may stay in content - strip all */
            .. Ammonia::default()
        };
    }
    let cleaned = String::from(TAGS2.clean(&a_buf));
    let matches: &[_] = &['\n', '\t', '\r'];
    cleaned.replace(matches, "").to_string()
}


#[test]
fn strip_html_tags_slice_test() {
    let a = "some skdnfdsfk<html><meta></meta><body></body></html> js\n\n\n\n\n\n\nn\\t\t\t\t\t\t\t\t\t\t\t aaaa bbbb cccc";
    let b = a.as_bytes();
    assert!(strip_html_tags_slice(b) == String::from("some skdnfdsfk jsn\\t aaaa bbbb cccc"), format!("Found {}", strip_html_tags_slice(b)))
}


fn process_file(abs_path: &str, f: &File) -> Result<FileEntry, String> {
    if valid_file_extensions(abs_path) {
        let bytes_to_read = 16384u64;
        let metadata = match f.metadata() {
            Ok(some) => some,
            Err(err) => return Err(format!("Failed to read metadata of path: {}. Cause: {}", abs_path, err)),
        };
        let mut reader = BufReader::new(f);

        match read_fragment(&mut reader, bytes_to_read) {
            Some(binary_content) => {
                match detect_encoding(&binary_content) {
                    Some(enc) => {
                        let an_owner = Owner {
                            origin: String::new(), /* XXX */
                            name: String::from(get_user_by_uid(metadata.uid()).unwrap().name()),
                            account_type: structs::AccountType::Regular,
                            uid: metadata.uid(),
                            gid: metadata.gid()
                        };
                        let buf = strip_html_tags(&binary_content);
                        let mut entry = structs::FileEntry {
                            owner: an_owner,
                            path: abs_path.to_string(),
                            size: metadata.size(),
                            mode: metadata.mode() as u32,
                            modified: get_time().sec - metadata.mtime(),
                            encoding: enc.name().to_string(),
                            .. Default::default()
                        };
                        match detect_language(&buf, Format::Text) {
                            (Some(Lang(lang)), Reliable) => {
                                entry.sha1 = sha1_of(buf);
                                entry.lang = String::from(lang);
                                debug!("Reliable detection: {}", json::encode(&entry).unwrap());
                                Ok(entry)
                            },

                            (Some(Lang(lang)), _) => {
                                entry.sha1 = sha1_of(buf);
                                entry.lang = String::from(lang);
                                debug!("Unreliable detection: {}", entry.to_string());
                                Ok(entry)
                            },

                            (None, _) => { /* not detected properly or value isn't reliable enough to tell */
                                entry.sha1 = sha1_of(buf);
                                entry.lang = String::from("en");
                                debug!("No detection for: {}. Doing fallback to 'en'", entry.to_string());
                                Ok(entry)
                            }
                        }
                    },

                    None => Err(String::from("None")),
                }
            },

            None =>
                Err(String::from(format!("Error reading file: '{}'", abs_path))),
        }
    } else {
        Err(String::from(format!("Invalid file type: '{}'", abs_path)))
    }
}


fn handle_file(path: &Path) -> Option<DomainEntry> {
    let name = match path.to_str() {
        Some(a_path) => a_path,
        None => "",
    };
    match File::open(name) {
        Ok(f) => {
            match process_file(name, &f) {
                Ok(file_entry) => {
                    /* default domain location: /home/{owner.name}/domains/{domain.name}/public_html/ */
                    let domain_from_path = Regex::new(r".*/domains/(.*)/public_html/.*").unwrap();
                    for _domain in domain_from_path.captures_iter(file_entry.path.as_str()) {
                        let domain = match _domain.at(1).unwrap() {
                            "" | "sharedip" | "default" | "suspended" => return None,
                            dom => dom,
                        };
                        let by = format!("{}/public_html/", domain);
                        debug!("Domain detection: {}", domain);

                        let request_path = file_entry.path.split(by.as_str()).last().unwrap_or("/");
                        let mut result = DomainEntry {
                            file: file_entry.clone(),
                            request_path: format!("/{}", request_path),
                            name: String::from(domain),
                            uuid: Uuid::new_v4(),
                            .. Default::default()
                        };

                        let request_protocols = vec!("http", "https");
                        for protocol in request_protocols {
                            let start = precise_time_ns();
                            match http::handle()
                                .follow_location(1)
                                .connect_timeout(3000)
                                .ssl_verifypeer(false)
                                .get(format!("{}://{}/{}", protocol, domain, request_path))
                                .exec() {
                                Ok(resp) => {
                                    let end = precise_time_ns();
                                    debug!("Processed external request: {}://{}/{} in {}ms", protocol, domain, request_path, (end - start) / 1000 / 1000);
                                    let contents = strip_html_tags_slice(resp.get_body());
                                    match protocol {
                                        "http" => {
                                            result.http_content_encoding = String::new(); /* XXX */
                                            result.http_content = contents.clone();
                                            result.http_content_size = contents.len();
                                            result.http_status_code = resp.get_code();
                                            result.http_response_time = (end - start) / 1000 / 1000;
                                        },
                                        "https" => {
                                            result.https_content_encoding = String::new(); /* XXX */
                                            result.https_content = contents.clone();
                                            result.https_content_size = contents.len();
                                            result.https_status_code = resp.get_code();
                                            result.https_response_time = (end - start) / 1000 / 1000;
                                        },
                                        _ => {
                                        }
                                    }
                                },
                                Err(err) => {
                                    // 2016-03-24 15:00:02 - dmilith - XXX: FIXME: NONDRY: UGLY:
                                    match protocol {
                                        "http" => {
                                            match err.to_string().as_str() {
                                                "Couldn't resolve host name" => {
                                                    debug!("{} host resolve problem: {:?}, for: {}", protocol, err, format!("{}://{}/{}", protocol, domain, request_path));
                                                    result.http_content_size = 0;
                                                    result.http_status_code = 410; /* http "gone" error - for unresolvable domain */
                                                },
                                                _ => {
                                                    debug!("{} host problem: {:?}, for: {} (404 fallback)", protocol, err, format!("{}://{}/{}", protocol, domain, request_path));
                                                    result.http_content_size = 0;
                                                    result.http_status_code = 404;
                                                }
                                            }
                                        },
                                        "https" => {
                                            match err.to_string().as_str() {
                                                "Couldn't resolve host name" => {
                                                    debug!("{} host resolve problem: {:?}, for: {}", protocol, err, format!("{}://{}/{}", protocol, domain, request_path));
                                                    result.https_content_size = 0;
                                                    result.https_status_code = 410; /* http "gone" error - for unresolvable domain */
                                                },
                                                _ => {
                                                    debug!("{} host problem: {:?}, for: {} (404 fallback)", protocol, err, format!("{}://{}/{}", protocol, domain, request_path));
                                                    result.https_content_size = 0;
                                                    result.https_status_code = 404;
                                                }
                                            }
                                        },
                                        _ => {
                                        }
                                    }
                                }
                            }
                        }
                        return Some(result)
                    };
                    None
                },
                Err(err) => {
                    if err.as_str().starts_with("Invalid file type") {
                        None /* report nothing */
                    } else { /* yell about everything else */
                        error!("Err processing file: {}, cause: {:?}", name, err);
                        None
                    }
                },
            }
        },
        Err(e) => {
            error!("Error in file: '{}', cause: {:?}.", name, e);
            None
        },
    }
}


fn read_path_from_env() -> String {
    let key = "HOME";
    let home = match env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("/home/"),
    };
    let key = "TRAV";
    match env::var(key) {
        Ok(val) => val,
        Err(_) => home, /* use ~ as fallback if no value of TRAV given */
    }
}


fn main() {
    env_logger::init().unwrap();

    let path = read_path_from_env();
    debug!("Traversing path: {:?}", path);

    let start = precise_time_ns();
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(5)
        .max_open(128)
        .into_iter();

    let mut files_processed = 0;
    let mut files_skipped = 0;
    for entry in walker /* filter everything we don't have access to */
                    .filter_map(|e| e.ok())
                    .filter(|e| e.metadata().unwrap().is_file() && e.path().to_str().unwrap_or("").contains("domains")) {
        match handle_file(entry.path()) {
            Some(entry_ok) => {
                let output_file = format!("/root/domain_${}_owner_{}.json", entry_ok.name, entry_ok.file.owner.name);
                let f = File::create(output_file.clone()).unwrap();
                let mut writer = BufWriter::new(f);
                writer.write(entry_ok.to_string().as_bytes()).unwrap();

                info!("DomainEntry: {} stored to file: {}", entry_ok, output_file);
                files_processed += 1;
            },
            None => {
                files_skipped += 1;
            },
        }
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
