use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Route {
    pub pattern: String,
    pub segments: Vec<Segment>,
    pub file_path: PathBuf,
    pub is_api: bool,
    pub is_dynamic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Static(String),
    Param(String),
}

#[derive(Clone, Default)]
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan_dir<P: AsRef<Path>>(&mut self, pages_dir: P) {
        self.routes.clear();
        let base = pages_dir.as_ref();
        if !base.exists() {
            return;
        }

        self.collect_routes(base, base);

        // Sort so static routes match before dynamic :param routes
        self.routes.sort_by(|a, b| {
            let a_has_param = a.segments.iter().any(|s| matches!(s, Segment::Param(_)));
            let b_has_param = b.segments.iter().any(|s| matches!(s, Segment::Param(_)));
            if a_has_param != b_has_param {
                if a_has_param {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            } else {
                b.segments.len().cmp(&a.segments.len())
            }
        });
    }

    fn collect_routes(&mut self, base: &Path, current: &Path) {
        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.collect_routes(base, &path);
                } else if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ext == "titanium" || ext == "ti" || ext == "cobalt" || ext == "co" || ext == "carbon" || ext == "cbn" || ext == "html" {
                        self.add_file_route(base, &path);
                    }
                }
            }
        }
    }

    fn add_file_route(&mut self, base: &Path, file_path: &Path) {
        let rel = file_path.strip_prefix(base).unwrap_or(file_path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let without_ext = rel_str.rsplit_once('.').map(|(p, _)| p).unwrap_or(&rel_str);

        let mut route_str = if without_ext == "index" {
            "/".to_string()
        } else if without_ext.ends_with("/index") {
            format!("/{}", &without_ext[..without_ext.len() - 6])
        } else {
            format!("/{}", without_ext)
        };

        if route_str.is_empty() {
            route_str = "/".to_string();
        }

        let is_api = route_str.starts_with("/api/");

        let mut segments = Vec::new();
        for seg in route_str.split('/').filter(|s| !s.is_empty()) {
            if seg.starts_with('[') && seg.ends_with(']') {
                let param_name = &seg[1..seg.len() - 1];
                segments.push(Segment::Param(param_name.to_string()));
            } else {
                segments.push(Segment::Static(seg.to_string()));
            }
        }

        let is_dynamic = segments.iter().any(|s| matches!(s, Segment::Param(_)));

        self.routes.push(Route {
            pattern: route_str,
            segments,
            file_path: file_path.to_path_buf(),
            is_api,
            is_dynamic,
        });
    }

    pub fn match_path(&self, request_path: &str) -> Option<(&Route, rhai::Map)> {
        let clean_path = if request_path.len() > 1 && request_path.ends_with('/') {
            &request_path[..request_path.len() - 1]
        } else {
            request_path
        };

        let req_segments: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();

        for route in &self.routes {
            if route.segments.is_empty() {
                if req_segments.is_empty() {
                    return Some((route, rhai::Map::new()));
                }
                continue;
            }

            if route.segments.len() != req_segments.len() {
                continue;
            }

            let mut params = rhai::Map::new();
            let mut matched = true;

            for (r_seg, &req_seg) in route.segments.iter().zip(req_segments.iter()) {
                match r_seg {
                    Segment::Static(s) => {
                        if s != req_seg {
                            matched = false;
                            break;
                        }
                    }
                    Segment::Param(param_name) => {
                        let decoded = urlencoding_decode(req_seg);
                        params.insert(param_name.clone().into(), rhai::Dynamic::from(decoded));
                    }
                }
            }

            if matched {
                return Some((route, params));
            }
        }

        None
    }
}

fn urlencoding_decode(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, _)| k.to_string())
        .next()
        .unwrap_or_else(|| s.to_string())
}
