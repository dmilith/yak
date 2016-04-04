use process::*;

use std::str::FromStr;
use std::error::Error;
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
        response.headers_mut().set(ContentType(content_type!(Text / Html; Charset = Utf8)));

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

/* HTTP path with params: /diff/:hostname/:username/:uuid1/:uuid2 */
fn chgset_diff_page(context: Context, response: Response) {
    let hostname = match context.variables.get("hostname") {
        Some(name) => name.to_string(),
        None => "s0".to_string(),
    };
    let username = match context.variables.get("username") {
        Some(name) => name.to_string(),
        None => "nobody".to_string(),
    };
    let uuid1 = match context.variables.get("uuid1") {
        Some(uuid) => Uuid::from_str(uuid.to_string().as_ref()).unwrap_or(root_failed_parse_from_string_to_uuid_uuid()),
        None => root_failed_no_uuud_given_uuid(),
    };
    let uuid2 = match context.variables.get("uuid2") {
        Some(uuid) => Uuid::from_str(uuid.to_string().as_ref()).unwrap_or(root_failed_parse_from_string_to_uuid_uuid()),
        None => root_failed_no_uuud_given_uuid(),
    };
    debug!("Params for chgset_diff_page: hn: {}, un: {}, uuid1: {}, uuid2: {}", hostname, username, uuid1, uuid2);

    let mut chsets = all_changesets(username.clone()).into_iter()
        .filter(|e| e.uuid == uuid1 || e.uuid == uuid2);

    let a = match chsets.next() {
        Some(next_one) => next_one,
        None => Changeset { uuid: root_invalid_uuid(), parent: root_invalid_uuid(), .. Default::default() },
    };
    let a_local_content: Vec<u8> = a.clone()
        .entries
        .into_iter()
        // .filter(|f| f.file.path.ends_with(".php") )
        .flat_map(|f| f.file.local_content )
        .collect();
    debug!("A: local content DBG: {:?}", String::from_utf8(a_local_content.clone()));

    let b = match chsets.next() {
        Some(next_one) => next_one,
        None => Changeset { uuid: root_invalid_uuid(), parent: root_invalid_uuid(), .. Default::default() },
    };
    let b_local_content: Vec<u8> = b.clone()
        .entries
        .into_iter()
        // .filter(|f| f.file.path.ends_with(".php") )
        .flat_map(|f| f.file.local_content )
        .collect();
    debug!("B local content DBG: {:?}", String::from_utf8(b_local_content.clone()));

    /*
        TODO:
            - local_content diff
            - http_content diff
            - https_content diff
            - encoding diff
            - language diff
    */

    print_difference(
        calculate_difference(
            String::from_utf8(a_local_content).unwrap(),
            String::from_utf8(b_local_content).unwrap(),
            ""));

    fn div(content: String) -> String {
        format!("<div class=\"item\">{}</div>", content)
    }
    response.send(format!("<html><body>{}{}</body></html>", div(a.to_string()), div(b.to_string())));
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
                "/history/:hostname/:username/:uuid1" => Get: Api(Some(chgset_history_page)),

                /* show details of given changeset. */
                "/chgset/:hostname/:username/:uuid1" => Get: Api(Some(chgset_show_page)),

                /* diff changesets with given uuids of specified user on specified host: */
                "/diff/:hostname/:username/:uuid1/:uuid2" => Get: Api(Some(chgset_diff_page)),

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
