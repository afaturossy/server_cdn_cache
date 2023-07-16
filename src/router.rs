use std::error::Error;
use std::path::PathBuf;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderName, Response};
use axum::Router;
use axum::routing::get;
use dotenvy::var;
use reqwest::{Client, header};
use tower_http::cors::CorsLayer;

use crate::error::MyError;

pub fn router() -> Router {
    Router::new()
        .route("/cdn/:url", get(cdn))
        .route("/image/:url", get(redirect_to))
        .layer(CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any))
}

fn create_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static(
        "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36"));

    reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build().expect("client error di buat")
}

async fn cdn(Path(url): Path<String>) -> Result<([(HeaderName, String); 2], Vec<u8>), MyError> {
    let client = create_client();
    let cache_name = md5::compute(&url);
    {}
    let cache_name = format!("{:?}", cache_name);

    let folder_path = var("FOLDER_CACHE").unwrap();
    let path = std::path::Path::new(&folder_path)
        .join(cache_name);

    let data_cache = tokio::fs::metadata(&path).await;


    match data_cache {
        Ok(v) => {
            let size = v.len();
            if v.is_file() && size > 200 {
                let image = std::fs::read(&path);

                let _ = filetime::set_file_atime(&path, filetime::FileTime::now());
                if image.is_ok() {
                    let image = image.unwrap();
                    return Ok(([(header::CONTENT_TYPE, "image/webp".to_string()), (header::CACHE_CONTROL, "public, max-age=604800".to_string())],
                               image));
                }
            } else {
                let image_webp = new_data(&client, &url, path).await;
                if image_webp.is_err() {
                    return Err(MyError::InternalServerError);
                }
                let image_webp = image_webp.unwrap();
                return Ok(([(header::CONTENT_TYPE, "image/webp".to_string()), (header::CACHE_CONTROL, "public, max-age=604800".to_string())],
                           image_webp));
            }
        }
        Err(_) => {
            let image_webp = new_data(&client, &url, path).await;
            if image_webp.is_err() {
                return Err(MyError::InternalServerError);
            }
            let image_webp = image_webp.unwrap();

            return Ok(([(header::CONTENT_TYPE, "image/webp".to_string()), (header::CACHE_CONTROL, "public, max-age=604800".to_string())],
                       image_webp));
        }
    }
    Err(MyError::BadRequest)
}


async fn new_data(client: &Client, url: &str, path: PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let host = reqwest::Url::parse(url)?;
    let host = host.host_str().unwrap_or("");

    let resp = client.get(url)
        .header(header::REFERER, host).send().await?;
    if resp.status() != 200 {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "error get image")));
    }

    let body = resp.bytes().await?;

    let body_copy = body.clone();
    let path_copy = path.clone();
    tokio::spawn(async move {
        std::fs::write(&path_copy, body_copy).ok();
    });

    Ok(body.to_vec())
}


async fn redirect_to(Path(url): Path<String>) -> Response<Body> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = axum::http::StatusCode::FOUND;

    resp.headers_mut().insert(header::LOCATION, url.parse().unwrap());
    resp
}


#[tokio::test]
async fn test() {}