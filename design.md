## TODO: Types:

### Changeset diff: (used in: fs_diffs ordered set/subset)
    * file_path - file path (file name preceded with path relative to domain dir, f.e.: "include/mypage.php")
    * file_info - file metadata structure (File::metadata())
    *

### enum Interpreters:
    * Php52
    * Php53
    * Php54
    * Php55
    * Php56
    * Php70
    * Python27
    * Python35
    * Perl
    * Bash/Dash/Sh/Csh/Zsh


### enum WebAppTypes:
    * WordPress
    * Joomla
    * Prestashop
    * Phpbb
    * Magento
    * Moodle
    * Owncloud
    * Drupal
    * Mybb
    * CmsMadeSimple
    * PhpFusion
    * Zurmo
    * Phpmyadmin
    * Afterlogic
    * Squirrelmail
    * Roundcube
    * Limesurvey
    * ZenCart
    * Piwik
    * PHPList
    * Mamboo
    * CustomX


### SeverityLevels:
    * Normal        - default mode
    * Pedantic      - + Warning = Suspected
                      + Empty = Broken
                      + Unresolvable = Broken
                      + Unknown = Broken
    * Psycho        - + Pedantic
                      + Suspected and Malicious are interpreted as Hacked
                      + each and single changeset includes full change information (not only changes which is default)


### DomainStates:
    * Ok            - web app looks fine
    * Warning       - minor issues
    * Suspected     - no malicious code found, but contains remains/and/or leftovers from hacks
    * Malicious     - web app file(s) match(es) contain(s) possibly dangerous/ malicious code
    * Hacked        - web app file(s) match(es) was scanned and has confirmed infections
    * Unresolvable  - no domain record, not registered, not bound to ips of server, etc.
    * Broken        - 401,403,404,500,501,502,503
    * Empty         - "white empty page" issue
    * Unknown       - no files found, no index.php


## SpiceStore:

* Current state stored in:
    `/Spice/HOST_NAME/USER_NAME/domains/DOMAIN_TLD.json`

* Changesets stored in:
    `/Spice/HOST_NAME/USER_NAME/domains/DOMAIN_TLD/.chsets/TIMESTAMP-SOME_UUID.chset`



## Changesets and history:

* Take initial state - taken from "current state" of domain in SpiceStore
* Check for diffs of previously checksummed files
* Create and store each changeset for later analysis
* (later) Get dynamic access to history of changes (built from stored changesets)


An example changesets:

```
"changeset": {
    "uuid": "UUID",
    "timestamp": "MILISECONDS-SINCE-1970",
    "owner": "USER_NAME",
    "domain.tld": {
        "environment": [list, of, changed, fields, or, values],
        "version": "10.2.3",
        "interpreter": "5.5",
        "outdated": false
    }
}
```

```
"changeset": {
    "uuid": "UUID",
    "timestamp": "MILISECONDS-SINCE-1970",
    "owner": "USER_NAME",
    "domain.tld": {
        "http_status": 401
        "environment": [list, of, changed, ini, fields, or, php, values],
        "fs_diffs": [list, of, modified, files, since, last, change],
        "fs_suspicious": [list, of, suspicious, files],
        "ini_used": "some/ambigous/path/because/why/not/php.ini",
    }
}
```


## Spice storehouse gathering task:

* TODO: Clean, fresh installations of web apps of each type (with initial db dumps)
* Learn sha, store it in json format:

```
"webapp_type_name": {
    "type": "Php",
    "version": "1.2.3",
    "interpreter": "5.3", // minimal version required by web app
    "supported": true, // if false, then app is a zombie with no support from devs
    "checksums": [
        "index.php": "sha1-of-index.php",
        "relative/file1.php": "sha1-of-file1.php",
        (..)
    ]
}
```

* Enter /home/USER_NAME/domains/DOMAIN_NAME
* For each domain perform 1 level deep scan
* Detect application:
  - types                       - set/subset of type WebAppTypes
  - states                      - set/subset of type DomainStates which determines domain state)
  - changesets                  - list of changesets made for this domain
  - version                     - version of detected web application
  - interpreter version         - version of interpreter to use with web application
  - predefined patterns         - patterns for common fuckups found on user accounts, that can be detected
  - default encodings           - encoding taken from db, interpreter and source of served file
  - external http status        - http error code (should be 200/301 most of the time)
  - framework base files checksums validation



## Data output format:

```
"USER_NAME": {
    "domains": ["domena1.tld", "domena2.tld"],
    "domena1.tld": {
        "type": "",
        "http_status": 200,
        "states": [domain, states, multiple, values, allowed],
        "changesets": [list, of, all, changeset, uuids, sorted, by, timestamp],
        "version": "",
        "outdated": false,
        "interpreter": "Php53",
        "environment": [list, of, all modified interpreter environment values
                        read from php.ini, htaccess and other bizzare places
                    ]
        "ini_used": "path/to/ini/file",
        "pecls_loaded": [..],
        "fs_diffs": [list, of, framework, files, that, have, checksum, diffs],
        "fs_suspicious": [list, of, suspicious, files, found by pattern match],
        "encoding_php": "",
        "encoding_db": "",
        "encoding_files": "",
    },
    "domena2.tld": {
        (...)
    }
}
```
