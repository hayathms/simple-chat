use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    Connect { username: String },
    SendMessage { username: String, message: String },
    Leave { username: String },
}

async fn handle_incoming_messages(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 10024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("Connection closed by server.");
                break;
            }
            Ok(n) => {
                if let Ok(msg) = String::from_utf8(buffer[..n].to_vec()) {
                    println!("\n[Chat] {}", msg);
                    print!("> ");
                    io::Write::flush(&mut io::stdout()).unwrap();
                }
            }
            Err(err) => {
                eprintln!("Error receiving message: {}", err);
                break;
            }
        }
    }
}

async fn handle_user_input(mut stream: TcpStream, username: &str) {

    loop {
        let mut stdin = async_std::io::stdin();
        let mut buffer = String::new();
        print!("> ");
        io::Write::flush(&mut io::stdout()).unwrap();
        stdin.read_line(&mut buffer).await.unwrap();
        let trimmed = buffer.trim();

        if trimmed == "leave" {
            let leave_message = ClientMessage::Leave { username: username.into() };
            let leave_data = serde_json::to_vec(&leave_message).unwrap();
            stream.write_all(&leave_data).await.unwrap();
            break;
        } else {
            let send_message = ClientMessage::SendMessage {
                username: username.into(),
                message: trimmed.to_string(),
            };
            let send_data = serde_json::to_vec(&send_message).unwrap();
            {
                stream.write_all(&send_data).await.unwrap();
                stream.write(b"\n").await.unwrap();
                stream.flush().await.unwrap();
            }
        }
    }
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let username = args.get(2).expect("Username required").to_string();
    let address = "127.0.0.1:8080";

    let mut stream = TcpStream::connect(address).await?;

    // Send connect message to the server
    let connect_message = ClientMessage::Connect { username: username.clone() };
    let connect_data = serde_json::to_vec(&connect_message).unwrap();
    stream.write_all(&connect_data).await.unwrap();
    stream.flush().await.unwrap();

    task::spawn(handle_incoming_messages(stream.clone()));
    handle_user_input(stream.clone(), &username).await;

    Ok(())
}

