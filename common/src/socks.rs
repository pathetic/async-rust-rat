use std::net::{Ipv6Addr, SocketAddrV6};
use std::io::*;
use net2::TcpStreamExt;
use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};

pub static MAGIC_FLAG : [u8;2] = [0x37, 0x37];

pub fn makeword(a : u8, b : u8) -> u16{
	((a as u16) << 8) | b as u16
}

#[derive(Debug, Clone)]
pub enum Addr {
	V4([u8; 4]),
	V6([u8; 16]),
	Domain(Box<[u8]>)
}

fn format_ip_addr(addr :& Addr) -> Result<String> {
	match addr {
		Addr::V4(addr) => {
			Ok(format!("{}.{}.{}.{}" , addr[0], addr[1] ,addr[2], addr[3]))
		},
		Addr::V6(addr) => {
			Ok(format!("{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}" , addr[0], addr[1] ,addr[2], addr[3], addr[4], addr[5] ,addr[6], addr[7] , addr[8], addr[9] ,addr[10], addr[11], addr[12], addr[13] ,addr[14], addr[15]))
		},
		Addr::Domain(addr) => match String::from_utf8(addr.to_vec()) {
			Ok(p) => Ok(p) ,
			Err(_e) => {
				Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid domain"))
			},
		}
	}
}

async fn tcp_transfer(stream : &mut TcpStream , addr : &Addr, address : &str , port :u16 ){
	let client  = match addr{
		Addr::V4(_) => {
			
			TcpStream::connect(address.to_owned()).await
		},
		Addr::V6(x) => {
			let ipv6 = Ipv6Addr::new(
				makeword(x[0] , x[1]) , 
				makeword(x[2] , x[3]) , 
				makeword(x[4] , x[5]) , 
				makeword(x[6] , x[7])  , 
				makeword(x[8] , x[9]) , 
				makeword(x[10] , x[11]) , 
				makeword(x[12] , x[13]) , 
				makeword(x[14] , x[15])
			);
			let v6sock = SocketAddrV6::new(ipv6 , port , 0 , 0 );
			TcpStream::connect(v6sock).await
		},
		Addr::Domain(_) => {
			TcpStream::connect(address.to_owned()).await
		}
	};

	let client = match client {
		Err(_) => {
			return;
		},
		Ok(p) => p
	};

	let raw_stream = client.into_std().unwrap();
	raw_stream.set_keepalive(Some(std::time::Duration::from_secs(10))).unwrap();
	let mut client = TcpStream::from_std(raw_stream).unwrap();

	let remote_port = client.local_addr().unwrap().port();

	let mut reply = Vec::with_capacity(22);
	reply.extend_from_slice(&[5, 0, 0]);

	match addr {
		Addr::V4(x) => {
			reply.push(1);
			reply.extend_from_slice(x);
		},
		Addr::V6(x) => {
			reply.push(4);
			reply.extend_from_slice(x);
		},
		Addr::Domain(x) => {
			reply.push(3);
			reply.push(x.len() as u8);
			reply.extend_from_slice(x);
		}
	}

	reply.push((remote_port >> 8) as u8);
	reply.push(remote_port as u8);

	if let Err(_e) = stream.write_all(&reply).await{
		return;
	};

	let mut buf1 = [0u8 ; 1024];
	let mut buf2 = [0u8 ; 1024];
	loop{
		tokio::select! {
			a = client.read(&mut buf1) => {

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
				match client.write_all(&buf2[..len]).await {
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
}

pub async fn socksv5_handle(mut stream: TcpStream) {
	loop {
		let mut header = [0u8 ; 2];
		if let Err(_e) = stream.read_exact(&mut header).await{
			break;
		};
		
		if header[0] != 5 {
			break;
		}
	
		let mut methods = vec![0u8; header[1] as usize].into_boxed_slice();
		if let Err(_e) = stream.read_exact(&mut methods).await{
			break;
		};
	
		if !methods.contains(&0u8) {
			break;
		}
	
		if let Err(_e) = stream.write_all(&[5, 0]).await{
			break;
		};

		let mut request =  [0u8; 4];
		if let Err(_e) = stream.read_exact(&mut request).await{
			break;
		};

		if request[0] != 5 {
			break;
		}
	
		let cmd = request[1];

		if cmd != 1 {
			break;
		}
	
		let addr = match request[3] {
			0x01 => {
				let mut ipv4 =  [0u8; 4];
				if let Err(_e) = stream.read_exact(&mut ipv4).await{
					break;
				};
				Addr::V4(ipv4)
			},
			0x04 => {
				let mut ipv6 =  [0u8; 16];
				if let Err(_e) = stream.read_exact(&mut ipv6).await{
					break;
				};
				Addr::V6(ipv6)
			},
			0x03 => {
				let mut domain_size =  [0u8; 1];
				if let Err(_e) = stream.read_exact(&mut domain_size).await{
					break;
				};
				let mut domain =  vec![0u8; domain_size[0] as usize].into_boxed_slice();
				if let Err(_e) = stream.read_exact(&mut domain).await{
					break;
				};

				Addr::Domain(domain)
			},
			_ => {
				break;
			}
		};
	
		let mut port = [0u8 ; 2];
		if let Err(_e) = stream.read_exact(&mut port).await{
			break;
		};

		let port = ((port[0] as u16) << 8) | port[1] as u16;
		let address_prefix = match format_ip_addr(&addr){
			Err(_) => {
				break;
			}
			Ok(p) => p
		};
		let address = format!("{}:{}" , address_prefix , port);

		if cmd == 1 {
			tcp_transfer(&mut stream , &addr , &address , port).await;
		}
		
	}
}