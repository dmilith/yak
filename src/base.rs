use uuid::Uuid;


pub fn root_uuid() -> Uuid {
    /* root */ Uuid::parse_str("4b84d962-ff55-4913-94fa-b20db7e1d2da").unwrap()
}
