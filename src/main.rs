use actix_multipart::Multipart;
use actix_web::{post, App, Error, HttpResponse, HttpServer};
use tokio::fs::File;
use futures_util::StreamExt as _;
use tokio::io::AsyncWriteExt;
use actix_files as fsa;
use std::fs;
use std::io::Cursor;
use zip_extract;
use std::path::PathBuf;

// TODO: Erorr handle, send proper HttpResponses
#[post("/push")]
async fn push(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // remove all files from /bin
    fs::remove_dir_all("./bin").unwrap();
    fs::create_dir("./bin").unwrap();
    
    // create new file to write payload to
    let mut file = File::create("./bin/packet.zip").await.unwrap();

    // read data stream and write to packet.zip in bin
    while let Some(item) = payload.next().await {
        let mut field = item?;

        while let Some(chunk) = field.next().await {
            let data = chunk?.to_vec();
            file.write_all(&data).await?;
        }
    }

    // extract packet.zip into /bin
    let bin = PathBuf::from("./bin"); 
    let archive: Vec<u8> = fs::read("./bin/packet.zip").unwrap();
    zip_extract::extract(Cursor::new(archive), &bin, true).unwrap();

    // remove packet.zip from /bin
    let _ = fs::remove_file("./bin/packet.zip");

    // Return a response
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fsa::Files::new("/static", "./bin").show_files_listing())
            .service(push)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}