use chrono::Utc;
use base64::Engine;
use serde::Deserialize;
use http::{Method, StatusCode};
use tower_http::cors::{Any, CorsLayer};
use std::{error::Error, path::PathBuf};
use axum::{ routing::{get, post}, Json, Router };
use tokio::{io::AsyncWriteExt, net::TcpListener};

//Structs
#[derive(Deserialize)]
struct ImageUpload {
    image_bytes: Vec<u8>,
    image_type: String,
}

//Functions
async fn check_folder_and_create(path:&PathBuf, ext:&str) -> Result<(), (StatusCode, String)> {

    let new_path = &path.join(&ext.to_uppercase());
    if !new_path.exists() {
        tokio::fs::create_dir_all(&new_path)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error while creating {} folder | Errorcode: {}", &ext.to_uppercase(), e)
                )
            })?;
    }

    Ok(())
}

async fn file_create_and_write(payload: &ImageUpload, folder_path: &PathBuf) -> Result<(), (StatusCode, String)> {
    
    let mut file = tokio::fs::File::create(folder_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while writing into file at {} | Errorcode: {}", &folder_path.to_string_lossy(), e)
            )
    })?;

    file.write_all(&payload.image_bytes).await.map_err(|e| {
        
         eprintln!("Error while writing image bytes into file | Errorcode: {}", e);
        
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write into file | Errorcode {}", e)
        )
    })?;

    Ok(())
}

//Handlers
async fn handle_get() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "server_running": "We Ball"
    }))
}

async fn handle_upload(Json(payload): Json<ImageUpload>) -> Result<(), (StatusCode, String)> {
    
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
    let folder_path = PathBuf::from("transferred_images");

    check_folder_and_create(&folder_path, &extension).await?;
    
    let file_name = format!("{}.{}", date_time, extension);
    let full_path:PathBuf = folder_path.join(extension.to_uppercase()).join(file_name);

    file_create_and_write(&payload, &full_path).await?;
    println!(
        "Saved to 📁 Folder: {:#?} ({} bytes)",
        &folder_path,
        payload.image_bytes.len()
    );

    Ok(())
}

async fn handle_fetch() -> Result<Json<serde_json::Value>, (StatusCode, String)> {

    let mut image_array: Vec<(String, String, Vec<u8>)> = vec![];
    let folder_path = PathBuf::from("transferred_images");

    if !folder_path.exists() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error {} folder does not exist | Errorcode: 404", &folder_path.to_string_lossy())
        ))
    };

    let mut entries = tokio::fs::read_dir(&folder_path)
        .await.map_err(|e| {(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read folders inside main folder | Errorcode: {}", e)
        )})?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to read files inside sub folder | Errorcode: {}", e)
    )})? {

        let path = entry.path();
        if entry.metadata().await.unwrap().is_dir() {
            println!("📁 Folder: {:#?}", path);

            let mut sub_entries = tokio::fs::read_dir(&path).await.unwrap();
            while let Some(sub_entry) = sub_entries.next_entry().await.unwrap() {
                let sub_path = sub_entry.path();

                println!("   └── Entry: {:#?}", sub_path);

                if sub_path.is_file() {
                    println!("       🖼️ File: {:#?}", sub_path);
                    println!("              File Name: {:?}", sub_path.file_name());
                    println!("              File Type: {:?}",
                        sub_path.extension().map(|ext| ext.to_string_lossy())
                    );

                    if let (Some(file_name), Some(exten)) =
                        (sub_path.file_name(), sub_path.extension())
                    {
                        let name = file_name.to_string_lossy().to_string();
                        let extension = exten.to_string_lossy().to_string();
                        let bytes = match tokio::fs::read(&sub_path).await {
                            Ok(bytes) => bytes,
                            Err(e) => return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Error while reading image bytes | Errorcode: {}", e)    
                            ))
                        };

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

//Main
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_check_folder_and_create() {
        
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_owned();

        println!("Temp dir: {:?}", test_path);

        let result = check_folder_and_create(&test_path, "JPG").await;
        assert!(result.is_ok(),  "Function returned error: {:?}", result);

        let jpg_path = test_path.join("JPG");
        println!("Checking path: {:?}", jpg_path); 
        println!("Directory exists: {}", jpg_path.exists());

        assert!(jpg_path.exists());
    }   

    #[tokio::test]
    async fn test_file_create_and_write() {
 
        let temp_dir = tempdir().unwrap();
        let temp_file = temp_dir.path().join("JPG").to_owned();
        let test_data = ImageUpload {
            image_type: "image/jpeg".to_string(),
            image_bytes: vec![0xFF, 0xD8, 0xFF]
        };

        let result = file_create_and_write(&test_data, &temp_file).await;

        assert!(result.is_ok());
        assert!(std::fs::read(&temp_file).is_ok());
        assert_eq!(std::fs::read(&temp_file).unwrap(), test_data.image_bytes);
    }
}