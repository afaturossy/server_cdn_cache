use dotenvy::{dotenv, var};

mod router;
pub mod error;

#[tokio::main]
async fn main() {
    //region ENV
    dotenv().expect("file .env tidak di temukan");
    let port = var("PORT").expect(".env PORT tidak di temukan");
    let folder = var("FOLDER_CACHE").expect(".env FOLDER_CACHE tidak di temukan");
    tokio::fs::create_dir_all(folder).await.expect("gagal create folder cache");
    //endregion

    let app = router::router();

    let addr = format!("0.0.0.0:{}", port);
    let addr_server = &addr.parse().unwrap();

    println!("SERVER RUNNING IN {}", addr);
    axum::Server::bind(addr_server)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
