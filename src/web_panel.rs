use process::*;

use std::error::Error;
// use std::sync::RwLock;
// use std::collections::btree_map::{BTreeMap, Iter};
use unicase::UniCase;

use rustful::{
    Server,
    Context,
    Response,
    Handler,
    TreeRouter
};
use rustful::header::{
    ContentType,
    AccessControlAllowOrigin,
    AccessControlAllowMethods,
    AccessControlAllowHeaders,
    Host
};
use rustful::StatusCode;


/* API endpoint with an optional action passed as a function: */
struct Api(Option<fn(Context, Response)>);

impl Handler for Api {
    fn handle_request(&self, context: Context, mut response: Response) {
        /* collect the accepted methods from the provided hyperlinks */
        let mut methods: Vec<_> = context.hyperlinks.iter().filter_map(|l| l.method.clone()).collect();
        methods.push(context.method.clone());

        /* setup cross origin resource sharing */
        response.headers_mut().set(AccessControlAllowOrigin::Any);
        response.headers_mut().set(AccessControlAllowMethods(methods));
        response.headers_mut().set(AccessControlAllowHeaders(vec![UniCase("content-type".into())]));

        if let Some(action) = self.0 {
            action(context, response);
        }
    }
}


fn index_page(context: Context, response: Response) {
    let person = match context.variables.get("person") {
        Some(name) => name,
        None => "stranger".into()
    };
    response.send(format!("Hello, {}!", person));
}


fn chgset_diff_page(context: Context, response: Response) {
    let user = match context.variables.get("username") {
        Some(name) => name,
        None => "nobody".into(),
    };
    let all = all_changesets(user.to_string())
        .into_iter()
        .map(|e| e.to_string() + "<br/><hr>")
        .collect::<String>();
    response.send(format!("<div>Listing timestamp sorted history:<br/>{}</div>", all));
}


fn chgset_show_page(context: Context, response: Response) {
    let user = match context.variables.get("username") {
        Some(name) => name,
        None => "nobody".into(),
    };
    let all = all_changesets(user.to_string())
        .into_iter()
        .map(|e| e.to_string() + "<br/><hr>")
        .collect::<String>();
    response.send(format!("<div>Listing timestamp sorted history:<br/>{}</div>", all));
}


fn chgset_history_page(context: Context, response: Response) {
    let user = match context.variables.get("username") {
        Some(name) => name,
        None => "nobody".into(),
    };
    let all = all_changesets(user.to_string())
        .into_iter()
        .map(|e| e.to_string() + "<br/><hr>")
        .collect::<String>();
    response.send(format!("<div>Listing timestamp sorted history:<br/>{}</div>", all));
}


/*
    Main Http server code:
 */
pub fn start() {
    let server_result = Server {
        host: root_default_http_port().into(), /*Turn a port number into an IPV4 host address (0.0.0.0:8080 in this case). */
        handlers: insert_routes!{
            /* route scenarios */
            TreeRouter::new() => {
                /* render history of all user changesets */
                "/history/:hostname/:username" => Get: Api(Some(chgset_history_page)),

                /* render history of given changeset of specified user on specified host */
                "/history/:hostname/:username/:uuid_ch1" => Get: Api(Some(chgset_history_page)),

                /* show details of given changeset. */
                "/chgset/:hostname/:username/:uuid_ch1" => Get: Api(Some(chgset_show_page)),

                /* diff changesets with given uuids of specified user on specified host: */
                "/diff/:hostname/:username/:uuid_ch1/:uuid_ch2" => Get: Api(Some(chgset_diff_page)),

                /* default route */
                "*" => Get: Api(Some(index_page)),
                Get: Api(Some(index_page)),
            }
        },

        ..Server::default()
    }.run();

    match server_result {
        Ok(_server) => {},
        Err(e) => error!("could not start server: {}", e.description())
    }
}
