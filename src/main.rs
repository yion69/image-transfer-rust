    use chrono::Utc;
    use http::{Method, StatusCode};
    use std::{error::Error, path::PathBuf};
    use serde::Deserialize;
    use tower_http::cors::{Any, CorsLayer};
    use axum::{ routing::{get, post}, Json, Router };
    use tokio::{ io::AsyncWriteExt, net::TcpListener};

    #[derive(Deserialize)]
    struct ImageUplaod {
        image_bytes: Vec<u8>,
        image_type: String
    }

    async fn handle_get() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "server_running": "We Ball"
        }))
    }

    async fn handle_upload (Json(payload): Json<ImageUplaod>) -> () {
    
        println!("image type => {} {} ", payload.image_type, payload.image_bytes.len());
        let extension = match payload.image_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "bin"
        };

        let date_time = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let mut folder_path = PathBuf::from("transferred_images");
        
        if !folder_path.exists() {
            tokio::fs::create_dir_all(&folder_path).await.map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, 
                    format!("Failed to create directory: {}", e))
                }).unwrap();
        }

        let file_name = format!("{}.{}", date_time, extension); 
        match extension {
            "jpg" => {
                if !folder_path.join("JPG").exists() {
                    tokio::fs::create_dir_all(&folder_path.join("JPG")).await.unwrap();
                }
                folder_path.push("JPG")
            },
            "png" => {
                if !folder_path.join("PNG").exists() {
                    tokio::fs::create_dir_all(&folder_path.join("PNG")).await.unwrap();
                }
                folder_path.push("PNG")
            },
            "gif" => {
                if !folder_path.join("GIF").exists() {
                    tokio::fs::create_dir_all(&folder_path.join("GIF")).await.unwrap();
                }
                folder_path.push("GIF")
            },
            "webp" => {
                if !folder_path.join("WEBP").exists() {
                    tokio::fs::create_dir_all(&folder_path.join("WEBP")).await.unwrap();
                }
                folder_path.push("WEBP")
            },
            _ => folder_path.push("Undefined"),
        }

        folder_path.push(file_name);

        println!("{}", folder_path.display());

        let mut file = tokio::fs::File::create(&folder_path).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write file: {}", e))
        }).unwrap();

        file.write(&payload.image_bytes).await.unwrap();

        println!("Saved {:?} ({} bytes)", &folder_path, payload.image_bytes.len());

    }

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn Error>> {
        
        let socket_address = "127.0.0.1:3000";
        let listener = TcpListener::bind(&socket_address).await?;

        let app:Router<()> = Router::new()
            .route("/", get(handle_get))
            .route("/upload", post(handle_upload))
            .layer(
                CorsLayer::new()
                    .allow_headers(Any)
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST])
            );
    
        println!("TCP Server Running on {}", socket_address);

        axum::serve(listener, app).await?;
        Ok(())
    }
