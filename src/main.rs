use axum::{ routing::{get, post}, Json, Router };
use base64::Engine;
use chrono::Utc;
use http::{Method, StatusCode};
use serde::Deserialize;
use std::{error::Error, path::PathBuf};
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct ImageUplaod {
    image_bytes: Vec<u8>,
    image_type: String,
}

async fn handle_get() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "server_running": "We Ball"
    }))
}

async fn handle_upload(Json(payload): Json<ImageUplaod>) -> Result<(), String> {
    println!(
        "image type => {} {} ",
        payload.image_type,
        payload.image_bytes.len()
    );
    let extension = match payload.image_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    };

    let date_time = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut folder_path = PathBuf::from("transferred_images");

    if !folder_path.exists() {
        tokio::fs::create_dir_all(&folder_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
            );
            format!("Failed to create directory: {}", e)
        })?;
    }

    let file_name = format!("{}.{}", date_time, extension);
    match extension {
        "jpg" => {
            if !folder_path.join("JPG").exists() {
                tokio::fs::create_dir_all(&folder_path.join("JPG"))
                    .await
                    .unwrap();
            }
            folder_path.push("JPG")
        }
        "png" => {
            if !folder_path.join("PNG").exists() {
                tokio::fs::create_dir_all(&folder_path.join("PNG"))
                    .await
                    .unwrap();
            }
            folder_path.push("PNG")
        }
        "gif" => {
            if !folder_path.join("GIF").exists() {
                tokio::fs::create_dir_all(&folder_path.join("GIF"))
                    .await
                    .unwrap();
            }
            folder_path.push("GIF")
        }
        "webp" => {
            if !folder_path.join("WEBP").exists() {
                tokio::fs::create_dir_all(&folder_path.join("WEBP"))
                    .await
                    .unwrap();
            }
            folder_path.push("WEBP")
        }
        _ => folder_path.push("Undefined"),
    }

    folder_path.push(file_name);

    println!("{}", folder_path.display());

    let mut file = tokio::fs::File::create(&folder_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
            );
            format!("Failed to write file: {}", e)
        })?;

    file.write(&payload.image_bytes).await.unwrap();

    println!(
        "Saved {:?} ({} bytes)",
        &folder_path,
        payload.image_bytes.len()
    );

    Ok(())
}

async fn handle_fetch() -> Result<Json<serde_json::Value>, String> {
    let mut image_array: Vec<(String, String, Vec<u8>)> = vec![];
    let folder_path = PathBuf::from("transferred_images");

    if !folder_path.exists() {
        return Err("Folder does not exist".to_string());
    };

    let mut entries = tokio::fs::read_dir(&folder_path)
        .await.map_err(|e| {
            format!("Failed to read the transferred folder for files or folder : {}", e)
        })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        format!("Failed to read next folder or file {}", e)
    })? {
        let path = entry.path();
        if entry.metadata().await.unwrap().is_dir() {
            println!("📁 Folder: {:#?}", path);

            let mut sub_entries = tokio::fs::read_dir(&path).await.unwrap();
            while let Some(sub_entry) = sub_entries.next_entry().await.unwrap() {
                let sub_path = sub_entry.path();

                println!("   └── Entry: {:#?}", sub_path);

                if sub_path.is_file() {
                    println!("       🖼️ File: {:#?}", sub_path);
                    println!("              File Name: {:#?}", sub_path.file_name());
                    println!(
                        "              File Type: {:#?}",
                        sub_path.extension().map(|ext| ext.to_string_lossy())
                    );

                    if let (Some(file_name), Some(exten)) =
                        (sub_path.file_name(), sub_path.extension())
                    {
                        let name = file_name.to_string_lossy().to_string();
                        let extension = exten.to_string_lossy().to_string();
                        let bytes = tokio::fs::read(&sub_path).await.unwrap();

                        image_array.push((
                            name,
                            extension,
                            base64::engine::general_purpose::STANDARD
                                .encode(bytes)
                                .into(),
                        ));
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "body": image_array
    })))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket_address = "127.0.0.1:3000";
    let listener = TcpListener::bind(&socket_address).await?;

    let app: Router<()> = Router::new()
        .route("/", get(handle_get))
        .route("/upload", post(handle_upload))
        .route("/fetch", get(handle_fetch))
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST]),
        );

    println!("TCP Server Running on {}", socket_address);

    axum::serve(listener, app).await?;
    Ok(())
}
