use std::error::Error;
use std::time::{Duration, SystemTime};

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

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(7 * 24 * 60 * 60)).await;
            let resp = remove_cache().await;
            if let Err(resp) = resp {
                println!("ERROR remove_cache->> {:?} ", resp);
            }
        }
    });

    let app = router::router();

    let addr = format!("0.0.0.0:{}", port);
    let addr_server = &addr.parse().unwrap();

    println!("SERVER RUNNING IN {}", addr);
    axum::Server::bind(addr_server)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn remove_cache() -> Result<(), Box<dyn Error>> {
    let folder = var("FOLDER_CACHE").unwrap();
    let path_folder = std::path::Path::new(&folder);

    let batas_waktu = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60); // 1 bulan
    

    if let Ok(entries) = std::fs::read_dir(path_folder) {
        for entry in entries.flatten() {

            let meta = entry.metadata()?;

            let last_access = meta.accessed()?;
            if last_access <= batas_waktu {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test(){
    remove_cache().await;
}