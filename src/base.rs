use uuid::Uuid;
use structs::*;


pub fn root_uuid() -> Uuid {
    /* root */ Uuid::parse_str("4b84d962-ff55-4913-94fa-b20db7e1d2da").unwrap()
}


pub fn root_invalid_uuid() -> Uuid {
    /* root */ Uuid::parse_str("deadbeef-ff55-4913-94fa-000000000000").unwrap()
}

pub fn root_failed_parse_from_string_to_uuid_uuid() -> Uuid {
    /* root */ Uuid::parse_str("FA170001-ff55-4913-94fa-000000000000").unwrap()
}


pub fn root_failed_no_uuud_given_uuid() -> Uuid {
    /* root */ Uuid::parse_str("FA170002-ff55-4913-94fa-000000000000").unwrap()
}


pub fn root_failed_no_next_item_uuid() -> Uuid {
    /* root */ Uuid::parse_str("FA170003-ff55-4913-94fa-000000000000").unwrap()
}


pub fn root_default_http_port() -> u16 {
    3000
}


pub fn root_default_connection_timeout() -> usize {
    2500
}


pub fn root_default_timeout() -> usize {
    5000
}


pub fn invalid_changeset() -> Changeset {
    Changeset{
        parent: root_invalid_uuid(),
        .. Default::default()
    }
}
