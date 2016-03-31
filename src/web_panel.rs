use std::error::Error;
use rustful::{Server, Context, Response, TreeRouter};


/*


HTTP PARAMS API use cases:

        http://localhost
                        /chgset/:username                     - shows history of all user changes between changesets
                        /chgset/:username/:uuid_ch1/:uuid_ch2 - shows difference between changeset uuid_ch1 and uuid_ch2
                        /chgset/:username/:timestamp          - shows all user changesets changes with exactly same or
                                                                bigger timestamp
                        /chgset/history/:username             - retrospective mode. Renders full user changeset history


 */


fn say_hello(context: Context, response: Response) {
    //Get the value of the path variable `:person`, from below.
    let person = match context.variables.get("person") {
        Some(name) => name,
        None => "stranger".into()
    };

    //Use the name from the path variable to say hello.
    response.send(format!("Hello, {}!", person));
}


pub fn start() {
    //Build and run the server.
    let server_result = Server {
        //Turn a port number into an IPV4 host address (0.0.0.0:8080 in this case).
        host: 3000.into(),

        //Create a TreeRouter and fill it with handlers.
        handlers: insert_routes!{
            TreeRouter::new() => {
                //Handle requests for root...
                Get: say_hello,

                //...and one level below.
                //`:person` is a path variable and it will be accessible in the handler.
                ":person" => Get: say_hello
            }
        },

        //Use default values for everything else.
        ..Server::default()
    }.run();

    match server_result {
        Ok(_server) => {},
        Err(e) => error!("could not start server: {}", e.description())
    }
}
