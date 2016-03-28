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
extern crate term;
extern crate difference;
extern crate flate2;
extern crate bincode;
extern crate rustc_serialize;
// extern crate flame;
// extern crate rsgenetic;

// #[macro_use] extern crate nickel;
// use nickel::Nickel;

use uuid::Uuid;
use time::precise_time_ns;
use walkdir::WalkDir;
use std::path::Path;

// local
mod structs;
mod utils;
mod base;
mod process;

use utils::*;
use structs::*;
use base::*;
use process::*;


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

        let mut changeset = Changeset {
            uuid: Uuid::new_v4(),
            parent: root_uuid(), // XXX - should be attached to "root branch"
            timestamp: time::precise_time_ns() / 1000 / 1000,
            entries: Vec::new(),
        };

        info!("Traversing path: '{}'", path);
        let walker = WalkDir::new(path)
            .follow_links(false)
            .max_depth(4)
            .max_open(512)
            .into_iter();

        for entry in walker /* filter everything we don't have access to */
                        .filter_map(|e| e.ok())
                        .filter(|e| e.metadata().unwrap().is_file() && e.path().to_str().unwrap_or("").contains("domains")) {

            // let entry_name = format!("path: {}", entry.path().to_str().unwrap_or("NO-FILE"));
            // flame::start(entry_name.clone());

            match process_domain(entry.path()) {
                Some(domain_entry) => {
                    /* write flamegraph */
                    // flame::end(entry_name.clone());
                    // let graph_file_name = format!("{}-{}.svg", user.name(), domain_entry.name);
                    // match flame::dump_svg(&mut File::create(graph_file_name).unwrap()) {
                    //     Ok(_) => debug!("Graph stored successfully"),
                    //     Err(err) => warn!("Failed to store graph: {}", err),
                    // }
                    // flame::clear();

                    changeset.entries.push(domain_entry);
                    files_processed += 1;
                },
                None => {
                    files_skipped += 1;
                },
            }
        }

        /* write changeset serialized to json */
        let (file_name, bytes_written) = store_changeset_json(user.name().to_string(), changeset.clone());
        info!("Changeset(json) stored: {} ({} bytes)", file_name, bytes_written);

        /* now write compressed binary changeset */
        let (file_name, bytes_written) = store_changeset(user.name().to_string(), changeset);
        info!("Changeset stored: {} ({} bytes)", file_name, bytes_written);
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
