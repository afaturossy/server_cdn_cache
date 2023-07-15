use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::HeaderName;
use axum::Router;
use axum::routing::get;
use dotenvy::var;
use reqwest::{Client, header};
use tower_http::cors::CorsLayer;

use crate::error::MyError;

pub fn router() -> Router {
    let client = reqwest::ClientBuilder::new()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::USER_AGENT, header::HeaderValue::from_static(
                "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36"));
            headers
        })
        .build().expect("client error di buat");

    let client = Arc::new(client);

    Router::new()
        .route("/cdn/:url", get(cdn))
        .with_state(client)
        .layer(CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any))
}


async fn cdn(State(client): State<Arc<Client>>, Path(url): Path<String>) -> Result<([(HeaderName, String); 2], Vec<u8>), MyError> {
    let cache_name = md5::compute(&url);
    {}
    let cache_name = format!("{:?}", cache_name);

    let folder_path = var("FOLDER_CACHE").unwrap();
    let path = std::path::Path::new(&folder_path)
        .join(cache_name);

    let data_cache = tokio::fs::metadata(&path).await;


    match data_cache {
        Ok(v) => {
            if v.is_file() {
                let image = tokio::fs::read(path).await;
                if image.is_ok() {
                    let image = image.unwrap();
                    return Ok(([(header::CONTENT_TYPE, "image/webp".to_string()), (header::CACHE_CONTROL, "public, max-age=604800, no-transform".to_string())],
                               image));
                }
            }
        }
        Err(_) => {
            let image_webp = new_data(&client, &url, path).await;
            if image_webp.is_err() {
                return Err(MyError::InternalServerError);
            }
            let image_webp = image_webp.unwrap();
            return Ok(([(header::CONTENT_TYPE, "image/avif".to_string()), (header::CACHE_CONTROL, "public, max-age=604800".to_string())],
                       image_webp));
        }
    }
    Err(MyError::BadRequest)
}


async fn new_data(client: &Client, url: &str, path: PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = client.get(url).send().await?;
    let body = resp.bytes().await?;

    //convert
    let s = std::time::Instant::now();
    let img = image::load_from_memory(&body)?;
    let convert = img.save_with_format(&path, image::ImageFormat::WebP);
    if convert.is_err() {
        let _ = std::fs::remove_file(&path);
    }
    let a = std::fs::read(path)?;
    let e = std::time::Instant::now();
    println!("{:?}", e - s);
    Ok(a)
}

#[tokio::test]
async fn test() {}