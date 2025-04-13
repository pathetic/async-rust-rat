use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    // This map holds the command senders for each connected client
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id = 0;

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        client_id += 1;

        let id = client_id;
        let (tx, rx) = mpsc::channel::<String>(32); // channel to send messages to this client
        clients.lock().unwrap().insert(id, tx.clone());

        let clients_clone = clients.clone();

        tokio::spawn(async move {
            handle_client(id, stream, rx, clients_clone).await;
        });
    }
}

// This function runs for each client
async fn handle_client(
    client_id: usize,
    stream: TcpStream,
    mut command_rx: mpsc::Receiver<String>,
    clients: Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>,
) {
    let (mut reader, mut writer) = tokio::io::split(stream);

    // Spawn a task to read from the socket
    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            let n = match reader.read(&mut buf).await {
                Ok(0) => break, // connection closed
                Ok(n) => n,
                Err(_) => break,
            };

            println!("Client {} says: {}", client_id, String::from_utf8_lossy(&buf[..n]));
        }
    });

    // Spawn a task to write to the socket based on channel input
    let write_task = tokio::spawn(async move {
        while let Some(msg) = command_rx.recv().await {
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    println!("Client {} disconnected", client_id);
    clients.lock().unwrap().remove(&client_id);
}
