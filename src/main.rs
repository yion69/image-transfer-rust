use std::error::Error;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{ TcpListener, TcpStream}};

async fn decode_stream (buffer: [u8; 1024]) -> String {

    unsafe  {
        let decoded = String::from_utf8_unchecked(buffer.to_vec());
        decoded 
    }

}

async fn handle_client (mut stream: TcpStream) -> std::io::Result<()> {
    
    let mut buffer = [0; 1024];

    if let Err(e) = stream.read(&mut buffer).await {
        eprintln!("Error HandleClient Read: {}", e)
    }

    if let Err(e) = &stream.write(&mut buffer).await {
        eprintln!("Error HandleClient Write: {}", e)
    }

    println!("Decoded Stream => {}", decode_stream(buffer).await);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let socket_address = "127.0.0.1:3000";
    let listener = TcpListener::bind(&socket_address).await?;
 
    println!("TCP Server Running on {}", socket_address);

    loop {
        let (stream,_) = listener.accept().await?;

        tokio::spawn( async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("{}", e);
            }
        });
    }
}
