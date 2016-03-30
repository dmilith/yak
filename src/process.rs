extern crate env_logger;

pub use base::*;
pub use utils::*;
pub use structs::*;

pub use std::env;
pub use uuid::Uuid;
pub use regex::Regex;
pub use std::path::Path;
pub use time::{get_time, precise_time_ns};
pub use std::io::{BufReader, BufWriter};
pub use std::fs::{remove_dir_all, create_dir_all, File, OpenOptions};
pub use std::io::prelude::{Read,Write};

pub use curl::http;
pub use users::get_user_by_uid;
pub use std::os::unix::fs::MetadataExt; /* Metadata trait */
pub use cld2::{detect_language, Format, Reliable, Lang};
pub use walkdir::WalkDir;

pub use flate2::write::ZlibEncoder;
pub use flate2::read::ZlibDecoder;
pub use flate2::Compression;

pub use rustc_serialize::json;
pub use bincode::SizeLimit;
pub use bincode::rustc_serialize::{encode, decode_from};


pub fn store_changeset_json(user_name: String, changeset: Changeset) -> (String, usize) {
    let changeset_dir = format!(".changesets/{}", user_name);
    match create_dir_all(changeset_dir.clone()) {
        Ok(_) => {},
        Err(err) => error!("{:?}", err),
    }
    let output_file = format!("{}/{}-{}.chgset.json", changeset_dir, changeset.uuid, changeset.timestamp);
    match OpenOptions::new()
                        .read(true)
                        .create(true)
                        .write(true)
                        .append(false)
                        .open(output_file.clone()) {
        Ok(f) => {
            let mut writer = BufWriter::new(f);
            match writer.write(changeset.to_string().as_bytes()) {
                Ok(bytes) => {
                    debug!("Changeset: {} has been stored in: {} ({} bytes)", changeset.uuid, output_file, bytes);
                    (output_file, bytes)
                },
                Err(err) => {
                    error!("Error: {}, file: {}", err, output_file);
                    (output_file, 0)
                }
            }
        },
        Err(err) => {
            error!("File open error: {}, file: {}", err, output_file);
            (output_file, 0)
        }
    }
}


pub fn store_changeset(user_name: String, changeset: Changeset) -> (String, usize) {
    let changeset_dir = format!(".changesets/{}", user_name);
    match create_dir_all(changeset_dir.clone()) {
        Ok(_) => {},
        Err(err) => error!("{:?}", err),
    }
    let file_name = format!("{}/{}-{}.chgset", changeset_dir, changeset.uuid, changeset.timestamp);
    let binary_encoded = encode(&changeset, SizeLimit::Infinite).unwrap();

    let mut zlib = ZlibEncoder::new(Vec::new(), Compression::Best);
    zlib.write(&binary_encoded[..]).unwrap();
    let compressed_bytes = zlib.finish().unwrap();

    let mut writer = BufWriter::new(File::create(file_name.clone()).unwrap());
    let bytes_written = writer.write(&compressed_bytes).unwrap();
    (file_name.to_string(), bytes_written)
}


pub fn all_changesets(user_name: String) -> Vec<Changeset> {
    let changeset_dir = format!(".changesets/{}", user_name);
    match create_dir_all(changeset_dir.clone()) {
        Ok(_) => {},
        Err(err) => error!("{:?}", err),
    }
    let mut changesets = vec!();
    info!("Reading changesets from dir: {}", changeset_dir);
    let walker = WalkDir::new(changeset_dir)
        .follow_links(false)
        .max_depth(2)
        .max_open(256)
        .into_iter();

    for entry in walker
        .filter_map(|e| e.ok())
        .filter(|e| e.path().to_str().unwrap_or("").ends_with(".chgset")) {

        match File::open(entry.path()) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut decoder = ZlibDecoder::new(reader);
                let changeset: Changeset = match decode_from(&mut decoder, SizeLimit::Infinite) {
                    Ok(r) => r,
                    Err(err) => {
                        let e = entry.path().to_str().unwrap_or("NO-FILE");
                        warn!("Data processing failure: {}, while processing changeset: {}", err, e);
                        invalid_changeset()
                    },
                };
                changesets.push(changeset.clone());
                debug!("Decoded Changeset: {}", changeset)
            },
            Err(err) => {
                error!("Failed to open file: {}. Error: {}", entry.path().to_str().unwrap_or("NO-FILE"), err);
            },
        }
    }
    changesets.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)); /* sort changesets by timestamp */
    changesets
}


pub fn all_changesets_json(user_name: String) -> Vec<Changeset> {
    let changeset_dir = format!(".changesets/{}", user_name);
    match create_dir_all(changeset_dir.clone()) {
        Ok(_) => {},
        Err(err) => error!("{:?}", err),
    }
    let mut changesets = vec!();
    info!("Reading changesets from dir: {}", changeset_dir);
    let walker = WalkDir::new(changeset_dir)
        .follow_links(false)
        .max_depth(2)
        .max_open(256)
        .into_iter();

    for entry in walker
        .filter_map(|e| e.ok())
        .filter(|e| e.path().to_str().unwrap_or("").ends_with(".chgset.json")) {

        match File::open(entry.path()) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut buf = String::new();
                let _ = reader.read_to_string(&mut buf);
                let changeset: Changeset = json::decode(buf.as_str()).unwrap();
                println!("{:?}", changeset.to_string());
                changesets.push(changeset.clone());
                debug!("Decoded Changeset: {}", changeset)
            },
            Err(err) => {
                error!("Failed to open file: {}. Error: {}", entry.path().to_str().unwrap_or("NO-FILE"), err);
            },
        }
    }
    changesets.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)); /* sort changesets by timestamp */
    changesets
}


pub fn mostrecent_changeset(user_name: String) -> Changeset {
    match all_changesets(user_name).pop() {
        Some(value) => value,
        None => invalid_changeset(),
    }
}


pub fn mostrecent_changeset_json(user_name: String) -> Changeset {
    match all_changesets_json(user_name).pop() {
        Some(value) => value,
        None => invalid_changeset(),
    }
}


pub fn process_file(abs_path: &str, f: &File) -> Result<FileEntry, String> {
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
                    account_type: AccountType::Regular,
                    uid: metadata.uid(),
                    gid: metadata.gid()
                };
                let buf = strip_html_tags(&binary_content);
                let mut entry = FileEntry {
                    owner: an_owner,
                    path: abs_path.to_string(),
                    /*
                        http://ół.pl/01ba0ee942dc3aefadcab35ebd5c9268.png
                        cut off all non printable control characters: */
                    local_content: binary_content.clone().into_iter().filter(|e| *e <= 13).collect::<Vec<u8>>(),
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


pub fn process_domain(path: &Path) -> Option<DomainEntry> {
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


#[cfg(test)]
mod tests {
    extern crate env_logger;
    use super::*;


    #[test]
    fn store_restore_changesets_json_test() {
        /* NOTE: you can put .changesets/ from any serve to /tmp/specials/S1 to process more "real life" examples */
        let specials = Path::new("/tmp/specials/S1");
        if specials.exists() {
            info!("Detected special specials dir! Yummy!");
            if env::set_current_dir(&specials).is_ok() {
                let walker = WalkDir::new(specials)
                    .follow_links(false)
                    .min_depth(2)
                    .max_depth(2)
                    .into_iter();

                for entry in walker
                    .filter_map(|e| e.ok()) {

                    let user = entry.path().to_str().unwrap().split("/").last().unwrap_or("");
                    if user.len() > 0 {
                        debug!("Processing JSON user: {}", user);
                        let all = all_changesets_json(String::from(user));
                        assert!(all.len() > 0, format!("Changeset dir empty for user: {}?", user));
                        let most_recent = mostrecent_changeset_json(String::from(user));
                        assert!(most_recent.timestamp > 10000000, "Timestamp is too small?");
                        assert!(most_recent.parent == root_uuid() || most_recent.parent == root_invalid_uuid());
                        debug!("Finished processing user: {}", user);
                    }
                }
            }
        }
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
            let entry = FileEntry {
                owner: Owner {
                    origin: String::new(), /* XXX */
                    name: String::from("root"),
                    account_type: AccountType::Admin,
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


    #[test]
    fn store_restore_changesets_test() {
        let origin_changeset = Changeset {
            uuid: Uuid::new_v4(),
            parent: root_uuid(), // XXX - should be attached to "root branch"
            timestamp: precise_time_ns() / 1000 / 1000,
            entries: vec!(
                DomainEntry {
                    file: FileEntry {
                        path: String::from("/tmp/index.php"),
                        local_content: "<?php echo INDEX phpinfo();".to_string().into_bytes(),
                        .. Default::default()
                    },
                    request_path: String::from("/index.php"),
                    name: String::from("index.php"),
                    .. Default::default()
                },
                DomainEntry {
                    file: FileEntry {
                        path: String::from("/tmp/main.php"),
                        local_content: "<?php echo MAIN phpinfo();".to_string().into_bytes(),
                        owner: Owner {
                            name: String::from("admin6"),
                            .. Default::default()
                        },
                        .. Default::default()
                    },
                    http_content: String::from("<?php echo MAIN phpinfo();"),
                    request_path: String::from("/main.php"),
                    name: String::from("main.php"),
                    .. Default::default()
                },
            ),
        };
        let mut changeset = origin_changeset.clone();
        changeset.uuid = Uuid::new_v4();
        changeset.timestamp += 1111;

        let root = Path::new("/tmp");
        if env::set_current_dir(&root).is_ok() {
            remove_dir_all(Path::new(".changesets")).unwrap_or(());
            store_changeset(String::from("admin6"), origin_changeset.clone());
            store_changeset(String::from("admin6"), changeset);
            let all = all_changesets(String::from("admin6"));
            assert!(all.len() == 2);
            let tsmp = mostrecent_changeset(String::from("admin6")).timestamp;
            assert!(tsmp == origin_changeset.timestamp + 1111, "Most recent timestamp isn't most recent?");
        }
        /* NOTE: you can put .changesets/ from any serve to /tmp/specials/S1 to process more "real life" examples */
        let specials = Path::new("/tmp/specials/S1");
        if specials.exists() {
            info!("Detected special specials dir! Yummy!");
            if env::set_current_dir(&specials).is_ok() {
                let walker = WalkDir::new(specials)
                    .follow_links(false)
                    .min_depth(2)
                    .max_depth(2)
                    .into_iter();

                for entry in walker
                    .filter_map(|e| e.ok()) {

                    let user = entry.path().to_str().unwrap_or("").split("/").last().unwrap_or("");
                    if user.len() > 0 {
                        debug!("Processing user: {}", user);
                        let all = all_changesets(String::from(user));
                        assert!(all.len() > 0, format!("Changeset dir empty for user: {}?", user));
                        let most_recent = mostrecent_changeset_json(String::from(user));
                        assert!(most_recent.timestamp > 10000000, "Timestamp is too small?");
                        assert!(most_recent.parent == root_uuid() || most_recent.parent == root_invalid_uuid());
                        debug!("Finished processing user: {}", user);
                    }
                }
            }
        }
    }

}
