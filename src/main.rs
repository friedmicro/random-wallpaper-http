#[macro_use]
extern crate rocket;

use rand::seq::SliceRandom;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;
use std::fs;
use std::path::Path;
use std::sync::Arc;

const PORT: u16 = 3500;
const ASSETS_DIR: &str = "./assets";

struct ImageAsset {
    path: String,
    dimension_x: u32,
    dimension_y: u32,
}

struct AppState {
    images: Vec<ImageAsset>,
}

#[derive(Deserialize)]
struct ImageRequest {
    min_x: Option<u32>,
    min_y: Option<u32>,
}

type Resp = Custom<(ContentType, Vec<u8>)>;

fn scan_assets(dir: &str) -> Vec<ImageAsset> {
    let mut images = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        eprintln!("Warning: cannot read image folder '{dir}'");
        return images;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let Ok((dx, dy)) = image::image_dimensions(&path) else {
            continue;
        };
        images.push(ImageAsset {
            path: path.to_string_lossy().into_owned(),
            dimension_x: dx,
            dimension_y: dy,
        });
    }
    images
}

fn select_image(
    images: &[ImageAsset],
    min_x: Option<u32>,
    min_y: Option<u32>,
) -> Option<&ImageAsset> {
    let filtered: Vec<&ImageAsset> = match (min_x, min_y) {
        (Some(mx), Some(my)) => images
            .iter()
            .filter(|i| i.dimension_x >= mx && i.dimension_y >= my)
            .collect(),
        _ => images.iter().collect(),
    };
    filtered.choose(&mut rand::thread_rng()).copied()
}

fn serve_entry(entry: &ImageAsset) -> Resp {
    match fs::read(&entry.path) {
        Ok(bytes) => Custom(Status::Ok, (content_type_for(&entry.path), bytes)),
        Err(_) => Custom(
            Status::NotFound,
            (
                ContentType::JSON,
                br#"{"error":"failed to read image file"}"#.to_vec(),
            ),
        ),
    }
}

fn error_response(msg: String) -> Resp {
    Custom(
        Status::NotFound,
        (
            ContentType::JSON,
            format!(r#"{{"error":"{msg}"}}"#).into_bytes(),
        ),
    )
}

#[post("/image", format = "json", data = "<req>")]
fn image_post(req: Json<ImageRequest>, state: &State<Arc<AppState>>) -> Resp {
    match select_image(&state.images, req.min_x, req.min_y) {
        Some(entry) => serve_entry(entry),
        None => error_response("no matching image".to_string()),
    }
}

fn content_type_for(path: &str) -> ContentType {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("jpg") | Some("jpeg") => ContentType::JPEG,
        Some("png") => ContentType::PNG,
        Some("gif") => ContentType::GIF,
        Some("webp") => ContentType::WEBP,
        Some("bmp") => ContentType::BMP,
        Some("svg") => ContentType::SVG,
        _ => ContentType::Binary,
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let images = scan_assets(ASSETS_DIR);
    if images.is_empty() {
        eprintln!("Warning: no images found in '{ASSETS_DIR}'");
    }
    eprintln!(
        "Loaded {} images from '{ASSETS_DIR}'. Serving on port {PORT}.",
        images.len()
    );

    let state = Arc::new(AppState { images });

    let figment = rocket::Config::figment()
        .merge(("port", PORT))
        .merge(("address", "0.0.0.0"));

    rocket::custom(figment)
        .manage(state)
        .mount("/", routes![image_post])
        .launch()
        .await?;

    Ok(())
}
