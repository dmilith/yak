use uuid::Uuid;


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
