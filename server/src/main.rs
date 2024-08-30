use async_std::io::Result;
use async_std::net::{TcpStream, TcpListener};
use async_std::stream::StreamExt;
use async_std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_std::io::ReadExt;
use async_std::io::WriteExt;


type UserStreamExt = Arc<Mutex<HashMap<String, TcpStream>>>;


#[async_std::main]
async fn main() -> Result<()>  {

    let user_clients: UserStreamExt = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!(" ------- main --- 1-----");
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        println!(" ------- main --- 2-----");
        let stream = stream?;
        handle_incoming_messages(stream, user_clients.clone()).await;
        println!(" ------- main --- 3-----");
    }
    Ok(())
}


#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    Connect { username: String },
    SendMessage { username: String, message: String },
    Leave { username: String },
}


async fn handle_incoming_messages(mut stream: TcpStream, user_clients: UserStreamExt) {
    println!(" ------- him --- 1-----");
    let mut buffer = vec![0u8; 100024];

     match stream.read_to_end(&mut buffer).await {
        Ok(_) => {
            println!(" ------- him --- 2-----");
            if let Ok(message) = serde_json::from_slice::<ClientMessage>(&buffer) {
                println!(" ------- him --- 3-----");
                println!("{:?}", message);
                match message {
                    ClientMessage::Connect { username } => {
                        println!(" ------- him --- 4-----");
                        let mut user_clients = user_clients.lock().await;
                        // user_clients.insert(username.clone(), stream.clone());
                    }
                    ClientMessage::SendMessage { username, message } => {
                        println!(" ------- him --- 5-----");
                        // let user_clients = user_clients.lock().await;
                        // for (user_name, mut userstream) in user_clients.iter() {
                        //     if user_name != &username {
                        //         userstream.write_all(message.as_bytes()).await.expect("Stream is dead");
                        //     }
                        // }
                    }
                    ClientMessage::Leave { username } => {
                        println!(" ------- him --- 6-----");
                        let mut user_clients = user_clients.lock().await;
                        user_clients.remove(&username);
                    }
                }
            }
            println!(" ------- him --- 7-----");
        }
        Err(_) => {
            println!(" ------- him --- 8-----");
        },
    }



    // loop {
    //     match stream.read(&mut buffer).await {
    //         Ok(0) => {
    //             println!("Connection closed by server.");
    //             break;
    //         }
    //         Ok(n) => {
    //             if let Ok(msg) = String::from_utf8(buffer[..n].to_vec()) {
    //                 println!("\n[Chat] {}", msg);
    //                 print!("> ");
    //                 io::Write::flush(&mut io::stdout()).unwrap();
    //             }
    //         }
    //         Err(err) => {
    //             eprintln!("Error receiving message: {}", err);
    //             break;
    //         }
    //     }
    // }
}



