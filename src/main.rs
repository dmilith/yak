#[macro_use] extern crate nickel;
extern crate walkdir;


use nickel::Nickel;
use walkdir::WalkDir;


fn main() {
    let mut server = Nickel::new();

    for entry in WalkDir::new(".") {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }

    server.utilize(router! {
        get "**" => |_req, _res| {
            "Hello world!"
        }
    });

    server.listen("127.0.0.1:6000");
}
