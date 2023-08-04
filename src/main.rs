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
    if let Err(e) = fs::remove_dir_all("./bin") {
        eprintln!("Error removing directory: {}", e);
        return Ok(HttpResponse::InternalServerError().finish());
    }
    if let Err(e) = fs::create_dir("./bin") {
        eprintln!("Error creating directory: {}", e);
        return Ok(HttpResponse::InternalServerError().finish());
    }

    // create new file to write payload to
    let mut file = match File::create("./bin/packet.zip").await {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating file: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    // read data stream and write to packet.zip in bin
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                eprintln!("Error reading field: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data.to_vec(),
                Err(e) => {
                    eprintln!("Error reading chunk: {}", e);
                    return Ok(HttpResponse::InternalServerError().finish());
                }
            };
            if let Err(e) = file.write_all(&data).await {
                eprintln!("Error writing data: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
        }
    }

    // extract packet.zip into /bin
    let bin = PathBuf::from("./bin"); 
    let archive = match fs::read("./bin/packet.zip") {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    if let Err(e) = zip_extract::extract(Cursor::new(archive), &bin, true) {
        eprintln!("Error extracting file: {}", e);
        return Ok(HttpResponse::InternalServerError().finish());
    }

    // remove packet.zip from /bin
    if let Err(e) = fs::remove_file("./bin/packet.zip") {
        eprintln!("Error removing file: {}", e);
        return Ok(HttpResponse::InternalServerError().finish());
    }

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