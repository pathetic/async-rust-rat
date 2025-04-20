use tokio::{io::AsyncReadExt, task, net::TcpStream};
use net2::TcpStreamExt;
use common::socks::MAGIC_FLAG;

pub struct ReverseProxy {
    pub ip: String,
    pub con_port: String,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl ReverseProxy {
    pub fn new() -> Self {
        Self { ip: "127.0.0.1".to_string(), con_port: "9876".to_string(), task: None }
    }

    pub fn setup(&mut self, ip: String, port: String) {
        self.ip = ip;
        self.con_port = port;
    }

    pub async fn start(&mut self) {
       let full_ip = format!("{}:{}", self.ip, self.con_port);

       let task = tokio::spawn(async move {
            let master_stream = match TcpStream::connect(full_ip.clone()).await{
                Ok(p) => p,
                Err(e) => {
                    println!("Error connecting to master: {}", e);
                    return;
                }
            };

            let raw_stream = master_stream.into_std().unwrap();
            raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
            let mut master_stream = TcpStream::from_std(raw_stream).unwrap();

            println!("Connected to master");

            loop {
                let mut buf = [0u8 ; 1];
                match master_stream.read_exact(&mut buf).await{
                    Err(e) => {
                        println!("Error reading from master: {}", e);
                        return;
                    },
                    Ok(p) => p
                };
    
                if buf[0] == MAGIC_FLAG[0] {
                    let stream = match TcpStream::connect(full_ip.clone()).await{
                        Err(e) => {
                            println!("Error connecting to client: {}", e);
                            return;
                        },
                        Ok(p) => p
                    };
    
                    let raw_stream = stream.into_std().unwrap();
                    raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
                    let stream = TcpStream::from_std(raw_stream).unwrap();
    
                    task::spawn(async {
                        common::socks::socksv5_handle(stream).await;
                    });
                }
            }
       });

       self.task = Some(task);
    }

    pub async fn stop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}


