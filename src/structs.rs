use uuid::Uuid;
use std::fmt;
use std::fmt::Display;
use rustc_serialize::{Decodable, Encodable, json};


#[derive(RustcDecodable, RustcEncodable, Debug)]
pub enum AccountType {
    Regular,
    Reseller,
    Managed,
    Admin,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct FileEntry {
    pub name: String,
    pub sha1: String,
    pub lang: String,
    pub encoding: String,
    pub size: i64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u16,
    pub modified: i64,
}


#[derive(RustcDecodable, RustcEncodable)]
pub struct DomainEntry {
    pub name: String,
    pub uuid: Uuid,
    pub file: FileEntry,
    pub http_content_encoding: String,
    pub http_content_size: i64,
    pub http_status_code: i64,
    pub request_time: i64,
    pub response_time: i64,
}

impl Default for FileEntry {
    fn default() -> FileEntry {
        FileEntry {
            name: String::new(),
            sha1: String::new(),
            lang: String::new(),
            encoding: String::new(),
            size: 0,
            uid: 0,
            gid: 0,
            mode: 0,
            modified: 0,
        }
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileEntry{{name: {}, sha1: {}, lang: {:?}, encoding: {}, size: {}, uid: {}, gid: {}, mode: {:o}, modified: {:?}s ago}}",
                self.name, self.sha1, self.lang, self.encoding, self.size,
                self.uid, self.gid, self.mode, self.modified,
        )
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
