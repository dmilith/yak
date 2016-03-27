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
use utils::*;
use structs::*;
use base::*;


fn process_file(abs_path: &str, f: &File) -> Result<FileEntry, String> {
    if valid_file_extensions(abs_path) {
        let bytes_to_read = 65535u64;
        let metadata = match f.metadata() {
            Ok(some) => some,
            Err(err) => return Err(format!("Failed to read metadata of path: {}. Cause: {}", abs_path, err)),
        };
        let mut reader = BufReader::new(f);

        match read_fragment(&mut reader, bytes_to_read) {
            Some(binary_content) => {
                let sys_pw = match get_user_by_uid(metadata.uid()) {
                    Some(user) => user,
                    None => get_user_by_uid(0).unwrap(), /* this user must exists */
                };
                let an_owner = Owner {
                    origin: String::new(), /* XXX */
                    name: String::from(sys_pw.name()),
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
                    .. Default::default()
                };
                match detect_encoding(&binary_content) {
                    Some(enc) => {
                        entry.encoding = enc.name().to_string();
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

                    None => {
                        entry.sha1 = sha1_of(buf);
                        entry.encoding = "ASCII".to_string();
                        entry.lang = "en".to_string();
                        Ok(entry)
                    },
                }
            },

            None =>
                Err(String::from(format!("Error reading file: '{}'", abs_path))),
        }
    } else {
        Err(String::from(format!("Invalid file type: '{}'", abs_path)))
    }
}


fn process_domain(path: &Path) -> Option<DomainEntry> {
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
                        let by = format!("{}/public_html", domain);
                        debug!("Domain detection: {}", domain);

                        let request_path = file_entry.path.split(by.as_str()).last().unwrap_or("/");
                        let mut result = DomainEntry {
                            file: file_entry.clone(),
                            request_path: format!("{}", request_path),
                            name: String::from(domain),
                            .. Default::default()
                        };

                        let request_protocols = vec!("http", "https");
                        for protocol in request_protocols {
                            let start = precise_time_ns();
                            match http::handle()
                                .follow_location(0)
                                .timeout(10000)
                                .connect_timeout(5000)
                                .ssl_verifypeer(false)
                                .get(format!("{}://{}{}", protocol, domain, request_path))
                                .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:10.0) Gecko/20100101 Firefox/10.0")
                                .exec() {
                                Ok(resp) => {
                                    let end = precise_time_ns();
                                    debug!("Processed external request: {}://{}{} in {}ms", protocol, domain, request_path, (end - start) / 1000 / 1000);
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
                                                    debug!("{} host resolve problem: {:?}, for: {}", protocol, err, format!("{}://{}{}", protocol, domain, request_path));
                                                    result.http_content_size = 0;
                                                    result.http_status_code = 410; /* http "gone" error - for unresolvable domain */
                                                },
                                                _ => {
                                                    debug!("{} host problem: {:?}, for: {} (404 fallback)", protocol, err, format!("{}://{}{}", protocol, domain, request_path));
                                                    result.http_content_size = 0;
                                                    result.http_status_code = 404;
                                                }
                                            }
                                        },
                                        "https" => {
                                            match err.to_string().as_str() {
                                                "Couldn't resolve host name" => {
                                                    debug!("{} host resolve problem: {:?}, for: {}", protocol, err, format!("{}://{}{}", protocol, domain, request_path));
                                                    result.https_content_size = 0;
                                                    result.https_status_code = 410; /* http "gone" error - for unresolvable domain */
                                                },
                                                _ => {
                                                    debug!("{} host problem: {:?}, for: {} (404 fallback)", protocol, err, format!("{}://{}{}", protocol, domain, request_path));
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
                Some(entry_ok) => {
                    let output_file = format!("{}_{}.json", entry_ok.file.owner.name, entry_ok.name);
                    match OpenOptions::new()
                                        .read(true)
                                        .create(true)
                                        .write(true)
                                        .append(true)
                                        .open(output_file.clone()) {
                        Ok(f) => {
                            let store_json = format!("json-file-dump: {}", output_file);
                            flame::start(store_json.clone());
                            let mut writer = BufWriter::new(f);
                            let entr = entry_ok.to_string() + ",";
                            match writer.write(entr.as_bytes()) {
                                Ok(some) => {
                                    info!("DomainEntry: {} has been stored in: {} ({} bytes)", entry_ok, output_file, some)
                                },
                                Err(err) => {
                                    error!("Error: {}, file: {}", err, output_file)
                                }
                            }
                            flame::end(store_json);
                        },
                        Err(err) => {
                            error!("File open error: {}, file: {}", err, output_file)
                        }
                    }

                    /* write flamegraph */
                    flame::end(entry_name.clone());
                    let graph_file_name = format!("{}-{}.svg", user.name(), entry_ok.name);
                    match flame::dump_svg(&mut File::create(graph_file_name).unwrap()) {
                        Ok(_) => info!("Graph stored successfully"),
                        Err(err) => error!("Failed to store graph: {}", err),
                    }
                    flame::clear();

                    files_processed += 1;
                },
                None => {
                    files_skipped += 1;
                },
            }
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
