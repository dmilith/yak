#![allow(dead_code)]
// TODO: just to silence crate level of warnings from code that's unused (yet)

use uuid::Uuid;
use std::fmt;
use std::fmt::Display;
use rustc_serialize::{Encodable, json};



#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub enum AccountType {
    Regular,
    Reseller,
    Managed,
    Admin,
}


#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Owner {
    pub name: String, /* user name */
    pub account_type: AccountType,
    pub origin: String, /* host origin name */
    pub uid: u32,
    pub gid: u32,
}


#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct FileEntry {
    pub path: String,
    pub sha1: String,
    pub lang: String,
    pub encoding: String,
    pub owner: Owner,
    pub size: i64,
    pub mode: u32,
    pub modified: i64,
}


#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct DomainEntry {
    pub name: String,
    pub request_path: String,
    pub file: FileEntry,

    pub http_content: String,
    pub http_content_encoding: String,
    pub http_content_size: usize,
    pub http_status_code: u32,
    pub http_response_time: u64,

    pub https_content: String,
    pub https_content_encoding: String,
    pub https_content_size: usize,
    pub https_status_code: u32,
    pub https_response_time: u64,

}


#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Changeset {
    pub uuid: Uuid,
    pub parent: Uuid,
    pub timestamp: u64,
    pub entries: Vec<DomainEntry>,
}


impl Default for Owner {
    fn default() -> Owner {
        Owner {
            name: String::new(),
            account_type: AccountType::Regular,
            origin: String::new(),
            uid: 0,
            gid: 0,
        }
    }
}


impl Default for FileEntry {
    fn default() -> FileEntry {
        FileEntry {
            path: String::new(),
            sha1: String::new(),
            lang: String::new(),
            encoding: String::new(),
            size: 0,
            owner: Owner {
                name: String::from("root"),
                origin: String::from("S0"),
                account_type: AccountType::Admin,
                uid: 0,
                gid: 0
            },
            mode: 0,
            modified: 0,
        }
    }
}


impl Display for Changeset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "{}", result),
            Err(err) => write!(f, "Failure serializing JSON for Changeset! Cause: {}", err)
        }
    }
}


impl Default for DomainEntry {
    fn default() -> DomainEntry {
        DomainEntry {
            name: String::from("localhost"),
            request_path: String::from("/"),
            file: FileEntry { .. Default::default() },
            http_content: String::new(),
            http_content_encoding: String::new(),
            http_content_size: 0,
            http_status_code: 0,
            http_response_time: 0,
            https_content: String::new(),
            https_content_encoding: String::new(),
            https_content_size: 0,
            https_status_code: 0,
            https_response_time: 0,
        }
    }
}


impl Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "{}", result),
            Err(err) => write!(f, "Failure serializing JSON for Owner! Cause: {}", err)
        }
    }
}


impl Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "{}", result),
            Err(err) => write!(f, "Failure serializing JSON for FileEntry! Cause: {}", err)
        }
    }
}


impl Display for DomainEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "{}", result),
            Err(err) => write!(f, "Failure serializing JSON for DomainEntry! Cause: {}", err)
        }
    }
}


enum DomainStates {
    Ok,
    Warning,
    Suspected,
    Malicious,
    Hacked,
    Unresolvable,
    Broken,
    Empty,
    Unknown,
}

enum WebAppTypes {
    WordPress,
    Joomla,
    Prestashop,
    Phpbb,
    Magento,
    Moodle,
    Owncloud,
    Drupal,
    Mybb,
    CmsMadeSimple,
    PhpFusion,
    Zurmo,
    Phpmyadmin,
    Afterlogic,
    Squirrelmail,
    Roundcube,
    Limesurvey,
    ZenCart,
    Piwik,
    PHPList,
    Mamboo,
    CustomX,
}

enum Interpreters {
    Php52,
    Php53,
    Php54,
    Php55,
    Php56,
    Php70,
    Python27,
    Python35,
    Perl,
    Shell,
    Text,
}
