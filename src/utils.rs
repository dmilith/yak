
use std::io::prelude::Read;
use ammonia::Ammonia;
use std::collections::HashSet;
use users::{User, AllUsers};
use regex::Regex;
use sha1;
use encoding::types::*;
use term;
use difference::diff;
use difference::Difference;
use encoding::all::*;


pub fn valid_file_extensions(name: &str) -> bool {
    lazy_static! {
        /*
        Regex will be compiled when it's used for the first time
        On subsequent uses, it will reuse the previous compilation
        */
        static ref RE: Regex = Regex::new(r"\.(php[0-9]*|[s]?htm[l0-9]*|txt|inc|py|pl|pm|rb|sh|[xyua]ml|htaccess|rss|[s]?css|js|mo|po|ini|ps|l?a?tex|svg)$").unwrap();
    }
    RE.is_match(name) || !name.contains(".")
}


#[cfg(test)]
#[test]
fn valid_file_extensions_test() {
    for valid in vec!(
        "somestrange123.file.php", ".htm", "a.txt", "file.html", "file.htm4",
        "exym.pl", "404.shtml", "album.rss", "a.ps", "a.latex", "mr.tex",
        "somefile", "SOMENOEXTFILE", "A", "a.php.txt.svg.xml.html.pl", "file.pm"
    ) {
        assert!(valid_file_extensions(valid), valid);
    }
    for invalid in vec!("file.plo", ".phpa", "file.pshtml", "f.pyc", "f.pod") {
        assert!(!valid_file_extensions(invalid), invalid);
    }
}


pub fn fetch_users() -> Vec<User> {
    let mut users: Vec<User> = unsafe { AllUsers::new() }.collect();
    users.sort_by(|a, b| a.name().cmp(&b.name()));
    users
}


pub fn sha1_of(input: String) -> String {
    let mut m = sha1::Sha1::new();
    m.update(input.as_bytes());
    m.hexdigest()
}


/* html tag cleaner PoC: */
pub fn strip_html_tags(binary_content: &Vec<u8>) -> String {
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


pub fn strip_html_tags_slice(binary_content: &[u8]) -> String {
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


#[cfg(test)]
#[test]
fn strip_html_tags_slice_test() {
    let a = "some skdnfdsfk<html><meta></meta><body></body></html> js\n\n\n\n\n\n\nn\\t\t\t\t\t\t\t\t\t\t\t aaaa bbbb cccc";
    let b = a.as_bytes();
    assert!(strip_html_tags_slice(b) == String::from("some skdnfdsfk jsn\\t aaaa bbbb cccc"), format!("Found {}", strip_html_tags_slice(b)))
}


pub fn detect_encoding(vec: &Vec<u8>) -> Option<EncodingRef> {
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


pub fn read_fragment<R>(reader: R, bytes_to_read: u64) -> Option<Vec<u8>> where R: Read {
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    match chunk.read_to_end(&mut buf) {
        Ok(_) =>
            Some(buf),
        _ =>
            None,
    }
}


pub fn calculate_difference(a: String, b: String, split: &str) -> Vec<Difference> {
    let (_dist, changes) = diff(a.as_ref(), b.as_ref(), split);
    changes
}


pub fn print_difference(changes: Vec<Difference>) {
    let mut t = term::stdout().unwrap();
    for c in changes.iter() {
        match c {
            &Difference::Same(ref z) => {
                t.fg(term::color::RED).unwrap();
                write!(t, "{}", z);
            },

            &Difference::Rem(ref z) => {
                t.fg(term::color::WHITE).unwrap();
                t.bg(term::color::RED).unwrap();
                write!(t, "{}", z);
                t.reset().unwrap();
            },

            _ => ()
        }
    }
    t.reset().unwrap();
    writeln!(t, "");

    for c in changes.iter() {
        match c {
            &Difference::Same(ref z) => {
                t.fg(term::color::GREEN).unwrap();
                write!(t, "{}", z);
            },

            &Difference::Add(ref z) => {
                t.fg(term::color::BLACK).unwrap();
                t.bg(term::color::GREEN).unwrap();
                write!(t, "{}", z);
                t.reset().unwrap();
            },

            _ => ()
        }
    }
    t.reset().unwrap();
    writeln!(t, "");
    t.flush().unwrap();
}
