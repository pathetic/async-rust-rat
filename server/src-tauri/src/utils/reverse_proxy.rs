use tokio::{io::{AsyncWriteExt, AsyncReadExt}, net::{TcpListener, TcpStream}};
use common::socks::MAGIC_FLAG;
use socket2::{Socket, TcpKeepalive};

pub async fn start_reverse_proxy(port: String, local_port: String) -> Option<tokio::task::JoinHandle<()>> {
    let master_addr = format!("{}:{}", "0.0.0.0", port);
    let socks_addr = format!("{}:{}", "0.0.0.0", local_port);
    
    let slave_listener = match TcpListener::bind(&master_addr).await{
        Err(_e) => {
            return None;
        },
        Ok(p) => p
    };

    let (slave_stream , _slave_addr) = match slave_listener.accept().await{
        Err(_e) => {
            return None;
        },
        Ok(p) => p
    };

    let raw_stream = slave_stream.into_std().unwrap();
    let socket = Socket::from(raw_stream);
    let keepalive = TcpKeepalive::new().with_time(std::time::Duration::from_secs(10));
    let _ = socket.set_tcp_keepalive(&keepalive);
    let mut slave_stream = TcpStream::from_std(socket.into()).unwrap();            
    
    let listener = match TcpListener::bind(&socks_addr).await{
        Err(_e) => {
            return None;
        },
        Ok(p) => p
    };

    let task = tokio::spawn(async move {
    loop {
        let (stream , _) = listener.accept().await.unwrap();

        let raw_stream = stream.into_std().unwrap();
        let socket = Socket::from(raw_stream);
        let keepalive = TcpKeepalive::new().with_time(std::time::Duration::from_secs(10));
        let _ = socket.set_tcp_keepalive(&keepalive);
        let mut stream = TcpStream::from_std(socket.into()).unwrap();

        if let Err(_e) = slave_stream.write_all(&[MAGIC_FLAG[0]]).await{
            break;
        };

        let (proxy_stream , _slave_addr) = match slave_listener.accept().await{
            Err(_e) => {
                return;
            },
            Ok(p) => p
        };

        let raw_stream = proxy_stream.into_std().unwrap();
        let socket = Socket::from(raw_stream);
        let keepalive = TcpKeepalive::new().with_time(std::time::Duration::from_secs(10));
        let _ = socket.set_tcp_keepalive(&keepalive);
        let mut proxy_stream = TcpStream::from_std(socket.into()).unwrap();


        let _task = tokio::spawn(async move {
            let mut buf1 = [0u8 ; 1024];
            let mut buf2 = [0u8 ; 1024];

            loop{
                tokio::select! {
                    a = proxy_stream.read(&mut buf1) => {
    
                        let len = match a {
                            Err(_) => {
                                break;
                            }
                            Ok(p) => p
                        };
                        match stream.write_all(&buf1[..len]).await {
                            Err(_) => {
                                break;
                            }
                            Ok(p) => p
                        };
    
                        if len == 0 {
                            break;
                        }
                    },
                    b = stream.read(&mut buf2) =>  { 
                        let len = match b{
                            Err(_) => {
                                break;
                            }
                            Ok(p) => p
                        };
                        match proxy_stream.write_all(&buf2[..len]).await {
                            Err(_) => {
                                break;
                            }
                            Ok(p) => p
                        };
                        if len == 0 {
                            break;
                        }
                    },
                }
            }
        });


    }
    });

    Some(task)
}
