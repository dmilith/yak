use uuid::Uuid;
use std::fmt;
use std::fmt::Display;
use rustc_serialize::{Decodable, Encodable, json};


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
    pub mode: u16,
    pub modified: i64,
}


#[derive(RustcDecodable, RustcEncodable)]
pub struct DomainEntry {
    pub name: String,
    pub uuid: Uuid,
    pub file: FileEntry,
    pub http_content_encoding: String,
    pub http_content_size: usize,
    pub http_status_code: u32,
    pub response_time: u64,
    // pub request_time: i64,
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


impl Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "Owner: {}", result),
            Err(err) => write!(f, "Failure serializing JSON for Owner! Cause: {}", err)
        }
    }
}


impl Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "FileEntry: {}", result),
            Err(err) => write!(f, "Failure serializing JSON for FileEntry! Cause: {}", err)
        }
    }
}


impl Display for DomainEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match json::encode(&self) {
            Ok(result) => write!(f, "DomainEntry: {}", result),
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


struct Changeset {
    uuid: Uuid,
}
