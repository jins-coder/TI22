use rhai::{Dynamic, Map};
use std::path::Path;

pub struct MediaUtils;

impl MediaUtils {
    pub fn file_info(path_str: &str) -> Map {
        let path = Path::new(path_str);
        let mut map = Map::new();

        if let Ok(metadata) = std::fs::metadata(path) {
            map.insert("exists".into(), Dynamic::from(true));
            map.insert("size_bytes".into(), Dynamic::from(metadata.len() as i64));
            map.insert(
                "extension".into(),
                Dynamic::from(
                    path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            );

            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            map.insert("mime".into(), Dynamic::from(mime));
        } else {
            map.insert("exists".into(), Dynamic::from(false));
            map.insert("size_bytes".into(), Dynamic::from(0i64));
            map.insert("extension".into(), Dynamic::from("".to_string()));
            map.insert("mime".into(), Dynamic::from("".to_string()));
        }

        map
    }

    pub fn to_data_uri(path_str: &str) -> String {
        let path = Path::new(path_str);
        if let Ok(bytes) = std::fs::read(path) {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            let b64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &bytes,
            );
            format!("data:{};base64,{}", mime, b64)
        } else {
            String::new()
        }
    }

    pub fn generate_identicon_svg(seed: &str, size: i64) -> String {
        let dim = if size <= 0 { 120 } else { size };
        let mut hash: u64 = 5381;
        for b in seed.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
        }

        let hue = hash % 360;
        let cell_size = dim / 5;
        let mut rects = String::new();

        for x in 0..3 {
            for y in 0..5 {
                let bit = (hash >> (x * 5 + y)) & 1;
                if bit == 1 {
                    let rx1 = x * cell_size;
                    let rx2 = (4 - x) * cell_size;
                    let ry = y * cell_size;

                    rects.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="hsl({}, 70%, 50%)"/>"#,
                        rx1, ry, cell_size, cell_size, hue
                    ));
                    if x != 2 {
                        rects.push_str(&format!(
                            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="hsl({}, 70%, 50%)"/>"#,
                            rx2, ry, cell_size, cell_size, hue
                        ));
                    }
                }
            }
        }

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}" style="background: rgba(0,0,0,0.2); border-radius: 8px;">{}</svg>"#,
            dim, dim, dim, dim, rects
        )
    }
}
