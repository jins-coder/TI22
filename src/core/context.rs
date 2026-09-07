use rhai::{Dynamic, Map};

#[derive(Clone, Debug)]
pub struct TitaniumRequest {
    pub method: String,
    pub path: String,
    pub query: Map,
    pub params: Map,
    pub body: Map,
    pub headers: Map,
}

impl TitaniumRequest {
    pub fn to_rhai_map(&self) -> Map {
        let mut map = Map::new();
        map.insert("method".into(), Dynamic::from(self.method.clone()));
        map.insert("path".into(), Dynamic::from(self.path.clone()));
        map.insert("query".into(), Dynamic::from(self.query.clone()));
        map.insert("params".into(), Dynamic::from(self.params.clone()));
        map.insert("body".into(), Dynamic::from(self.body.clone()));
        map.insert("headers".into(), Dynamic::from(self.headers.clone()));
        map
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum TitaniumResponse {
    Html {
        status: u16,
        body: String,
    },
    Json {
        status: u16,
        data: Dynamic,
    },
    Redirect {
        status: u16,
        location: String,
    },
    Sse {
        status: u16,
        body: String,
    },
    Raw {
        status: u16,
        content_type: String,
        bytes: Vec<u8>,
    },
}
