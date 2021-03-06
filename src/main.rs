#[macro_use]
extern crate rustful;

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
extern crate rayon;
extern crate unicase;
// extern crate flame;
// extern crate rsgenetic;


// local
mod structs;
mod utils;
mod base;
mod process;
mod api_server;

use process::*;
use api_server::start;

use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};


fn main() {
    env_logger::init().unwrap();

    for arg in env::args() {
        match arg.as_str() {
            "api" | "www" | "web" | "server" | "s" => {
                info!("Starting Http service on port: {}", root_default_http_port());
                api_server::start();
            },

            _ => {},
        }
    }

    info!("Traversing home dirs..");
    main_traverser()
}


fn main_traverser() {
    let start = precise_time_ns();
    let files_processed: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let files_skipped: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    // let _ = rayon::Configuration::new().set_num_threads(4);

    fetch_users().par_iter_mut().for_each(
        |user| {
            let path = format!("/home/{}/", user.name());
            if Path::new(path.as_str()).exists() {
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

                            let value = files_processed.load(Ordering::SeqCst);
                            files_processed.store(value + 1, Ordering::SeqCst);
                        },
                        None => {
                            let value = files_skipped.load(Ordering::SeqCst);
                            files_skipped.store(value + 1, Ordering::SeqCst);
                        },
                    }
                }

                // /* write changeset serialized to json */
                // let (file_name, bytes_written) = store_changeset_json(user.name().to_string(), changeset.clone());
                // info!("Changeset(json) stored: {} ({} bytes)", file_name, bytes_written);

                /* now write compressed binary changeset */
                let (file_name, bytes_written) = store_changeset(user.name().to_string(), changeset);
                info!("Changeset stored: {} ({} bytes)", file_name, bytes_written);
            }
        }
    );

    let end = precise_time_ns();
    info!("Traverse for: {} files, (skipped: {} files), elapsed: {} miliseconds", files_processed.load(Ordering::SeqCst), files_skipped.load(Ordering::SeqCst), (end - start) / 1000 / 1000);

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
