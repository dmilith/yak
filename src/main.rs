#[macro_use] extern crate lazy_static; // ensure that regular expressions are compiled exactly once
extern crate regex;

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

use uuid::Uuid;
use regex::Regex;
use core::result::Result;
use time::*;
use ammonia::*;
use cld2::{detect_language, Format, Reliable, Lang};
use encoding::*;
use encoding::all::*;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::collections::{HashSet};
use std::os::unix::fs::MetadataExt;


mod structs;
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
    TAGS.clean(&a_buf)
}


fn process_file(name: &str, f: &File) -> Result<FileEntry, String> {
    if valid_file_extensions(name) {
        let bytes_to_read = 16384u64;
        let metadata = f.metadata().unwrap();
        let mut reader = BufReader::new(f);

        match read_fragment(&mut reader, bytes_to_read) {
            Some(binary_content) => {
                match detect_encoding(&binary_content) {
                    Some(enc) => {
                        let buf = strip_html_tags(&binary_content);
                        match detect_language(&buf, Format::Text) {
                            (Some(Lang(lang)), Reliable) => {
                                let entry = structs::FileEntry {
                                    name: name.to_string(),
                                    sha1: sha1_of(buf),
                                    lang: lang.to_string(),
                                    encoding: enc.name().to_string(),
                                    size: metadata.size(),
                                    uid: metadata.uid(),
                                    gid: metadata.gid(),
                                    mode: metadata.mode(),
                                    modified: get_time().sec - metadata.mtime()
                                };
                                println!("Reliable detection: {}", entry.to_string());
                                Ok(entry)
                            },

                            (Some(Lang(lang)), _) => {
                                let entry = structs::FileEntry {
                                    name: name.to_string(),
                                    sha1: sha1_of(buf),
                                    lang: lang.to_string(),
                                    encoding: enc.name().to_string(),
                                    size: metadata.size(),
                                    uid: metadata.uid(),
                                    gid: metadata.gid(),
                                    mode: metadata.mode(),
                                    modified: get_time().sec - metadata.mtime()
                                };
                                println!("Unreliable detection: {}", entry.to_string());
                                Ok(entry)
                            },

                            (None, Reliable) => {
                                let entry = structs::FileEntry {
                                    name: name.to_string(),
                                    sha1: sha1_of(buf),
                                    lang: String::new(),
                                    encoding: enc.name().to_string(),
                                    size: metadata.size(),
                                    uid: metadata.uid(),
                                    gid: metadata.gid(),
                                    mode: metadata.mode(),
                                    modified: get_time().sec - metadata.mtime()
                                };
                                println!("Reliable no detection: {}", entry.to_string());
                                Ok(entry)
                            },

                            (None, _) => { /* not detected properly or value isn't reliable enough to tell */
                                let entry = structs::FileEntry {
                                    name: name.to_string(),
                                    sha1: sha1_of(buf),
                                    lang: String::new(),
                                    encoding: enc.name().to_string(),
                                    size: metadata.size(),
                                    uid: metadata.uid(),
                                    gid: metadata.gid(),
                                    mode: metadata.mode(),
                                    modified: get_time().sec - metadata.mtime()
                                };
                                println!("Unreliable no detection: {}", entry.to_string());
                                Ok(entry)
                            }
                        }
                    },

                    None => Err(String::from("None")),
                }
            },

            None =>
                Err(String::from("Error reading file!")),
        }
    } else {
        Err(String::from("None"))
    }
}


fn handle_file(path: &Path) {
    let name = path.to_str().unwrap();

    match File::open(name) {
        Ok(f) => {
            match process_file(name, &f) {
                Ok(val) => {
                    println!("Ok: {}", val.to_string())
                },
                Err(err) => {
                    println!("Err: {}", err)
                },
            }
        },
        Err(e) => println!("Error in file IO: {:?}", e),
    }
}


fn read_path_from_env() -> String {
    let key = "HOME";
    let home = match env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("/tmp/"),
    };
    let key = "TRAV";
    match env::var(key) {
        Ok(val) => val,
        Err(_) => home, /* use ~ as fallback if no value of TRAV given */
    }
}


fn main() {
    let text = "Młody Amadeusz szedł suchą szosą.";
    println!("{:?}", detect_language(text, Format::Text));

    let path = read_path_from_env();
    println!("Traversing path: {:?}", path);

    let start = precise_time_ns();
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(3)
        .max_open(128)
        .into_iter();

    let mut files_processed = 0;
    for entry in walker.filter_map(|e| e.ok()) { /* filter everything we don't have access to */
        if entry.file_type().is_file() {
            handle_file(entry.path());
            files_processed = files_processed + 1;
        }
    }
    let end = precise_time_ns();
    println!("Traverse for {} items, took: {} ms to complete", files_processed, (end - start) / 1000 / 1000);

    // let mut server = Nickel::new();
    // server.utilize(router! {
    //     get "**" => |_req, _res| {
    //         "Hello world!"
    //     }
    // });
    // server.listen("127.0.0.1:6000");
}
