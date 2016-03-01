// #[macro_use] extern crate nickel;
// use nickel::Nickel;
extern crate walkdir;
extern crate cld2;
extern crate encoding;

use cld2::{detect_language, Format, Reliable, Lang};
use encoding::*;
use encoding::all::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use walkdir::*; //{DirEntry, WalkDir, WalkDirIterator};


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

fn handle_file(path: &Path) {
    let bytes_to_read = 8192u16;
    let name = path.to_str().unwrap();
    let f = File::open(name).unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    if  name.ends_with(".php") ||
        name.ends_with(".txt") ||
        name.ends_with(".py") ||
        name.ends_with(".pl") {

        match reader.read_until(bytes_to_read as u8, &mut buffer) {
            Ok(size) => {
                match detect_encoding(&mut buffer) {
                    Some(enc) => {
                        let buf = String::from_utf8_lossy(&mut buffer);
                        match detect_language(&buf, Format::Text) {
                            (Some(Lang(lang)), Reliable) =>
                                println!("Reliable detection: {}, lang: {:?}, encoding: {}, size: {}",
                                    name, lang, enc.name(), size),

                            (Some(Lang(lang)), _) =>
                                println!("Unreliable detection: {}, lang: {:?}, encoding: {}, size: {}",
                                    name, lang, enc.name(), size),

                            (None, Reliable) =>
                                println!("Reliable no detection: {}, lang: Unknown, encoding: {}, size: {}",
                                    name, enc.name(), size),

                            (None, _) => /* not detected properly or value isn't reliable enough to tell */
                                println!("Unreliable no detection: {}, lang: Unknown, encoding: {}, size: {}",
                                    name, enc.name(), size),
                        }
                    },

                    None => println!("None"),
                }
            },

            Err(err) =>
                println!("Error reading file! {:?}", err),
        }
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
        Err(_) => home, /* use ~ as fallback if no value of TRAVERSE given */
    }
}


fn main() {
    let text = "Młody Amadeusz szedł suchą szosą.";
    println!("{:?}", detect_language(text, Format::Text));

    let path = read_path_from_env();
    println!("Traversing path: {:?}", path);
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(3)
        .max_open(128)
        .into_iter();

    for entry in walker.filter_map(|e| e.ok()) { /* filter everything we don't have access to */
        if entry.file_type().is_file() {
            handle_file(entry.path());
        }
    }

    // let mut server = Nickel::new();
    // server.utilize(router! {
    //     get "**" => |_req, _res| {
    //         "Hello world!"
    //     }
    // });
    // server.listen("127.0.0.1:6000");
}
