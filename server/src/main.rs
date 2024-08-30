use async_std::io::prelude::*;
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::Mutex;
use async_std::task;
use serde::{Deserialize, Serialize};
use serde_json;
use async_std::sync::Arc;
use async_std::io::Result;
use std::collections::HashMap;


#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    Connect { username: String },
    SendMessage { username: String, message: String },
    Leave { username: String },
}

type UserStreamExt = Arc<Mutex<HashMap<String, TcpStream>>>;

async fn handle_client(stream: Arc<Mutex<TcpStream>>, addr: std::net::SocketAddr, user_clients: UserStreamExt) {
    let mut buffer = vec![0u8; 1024];
    loop {
        let mut stream = stream.lock().await; // Lock the async mutex to get mutable access to the stream

        println!("Waiting for data from {}", addr);
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                println!("Connection closed by {}", addr);
                return;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from {}: {:?}", addr, e);
                return;
            }
        };

        let received_data = &buffer[..n];

        match serde_json::from_slice::<ClientMessage>(received_data) {
            Ok(message) => {
                match message {
                    ClientMessage::Connect { username } => {
                        let mut user_clients = user_clients.lock().await;
                        if user_clients.contains_key(&username) {
                            let msg = "Username already exists".to_string();
                            stream.write_all(msg.as_bytes()).await.unwrap();
                            stream.flush().await.unwrap();
                            panic!("Username already exists");
                        }
                        user_clients.insert(username.clone(), stream.clone());
                        println!("{} has connected", username);
                    }
                    ClientMessage::SendMessage { username, message } => {
                        let user_clients = user_clients.lock().await;
                        for (user, mut stream) in user_clients.iter() {
                            // skip if user is the sender
                            if user != &username {
                                let msg = format!("{}: {}", username, message);
                                stream.write_all(msg.as_bytes()).await.unwrap();
                                stream.flush().await.unwrap();
                            }
                        }
                    }
                    ClientMessage::Leave { username } => {
                        let mut user_clients = user_clients.lock().await;
                        user_clients.remove(&username);
                        println!("{} has left", username);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize message: {:?}", e);
            }
        }
    }
}

async fn run_server(address: &str) -> std::io::Result<()> {

    let user_clients: UserStreamExt = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind(address).await?;
    println!("Server is running on {}", address);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        let stream = Arc::new(Mutex::new(stream)); // Wrap the stream in Arc<async_std::sync::Mutex>
        task::spawn(handle_client(Arc::clone(&stream), addr, user_clients.clone())); // Spawn the client handler
    }
}

fn main() -> std::io::Result<()> {
    task::block_on(run_server("127.0.0.1:8080"))
}
