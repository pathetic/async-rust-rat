use std::marker::PhantomData;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::packets::*;

use rand::RngCore;
use rand_chacha::ChaCha20Rng;

use encryption::*;

pub struct Connection<I, O> {
    stream: TcpStream,
    _marker: PhantomData<(I, O)>,
}

pub struct ConnectionReader<P: Packet> {
    stream: OwnedReadHalf,
    buffer: BytesMut,
    _marker: PhantomData<P>,
}

pub struct ConnectionWriter<P: Packet> {
    stream: BufWriter<OwnedWriteHalf>,
    _marker: PhantomData<P>,
}

impl<I, O> Connection<I, O>
where
    I: Packet,
    O: Packet,
{
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: PhantomData,
        }
    }

    pub fn split(self) -> (ConnectionReader<I>, ConnectionWriter<O>) {
        let (read, write) = self.stream.into_split();
        let read = ConnectionReader::<I> {
            stream: read,
            buffer: BytesMut::with_capacity(4096),
            _marker: PhantomData,
        };
        let write = ConnectionWriter::<O> {
            stream: BufWriter::new(write),
            _marker: PhantomData,
        };
        (read, write)
    }
}

impl<P: Packet> ConnectionReader<P> {
    pub async fn read_packet(
        &mut self,
        secret: &Option<Vec<u8>>,
        nonce_generator: Option<&mut ChaCha20Rng>,
    ) -> Result<Option<P>, String> {
        let secret_and_nonce = if let Some(secret) = secret {
            let mut buf = [0u8; crate::SECRET_LEN];
            buf.copy_from_slice(&secret[..]);
            let mut nonce = [0u8; crate::NONCE_LEN];
            nonce_generator
                .expect("Expected `nonce_generator` to be `Some` because `secret` was `Some`.")
                .fill_bytes(&mut nonce);
            Some((buf, nonce))
        } else {
            None
        };
        loop {
            if let Some((secret, nonce)) = secret_and_nonce {
                if let Ok((p, b)) =
                    decrypt_frame(&mut self.buffer.as_ref(), &secret, &nonce)
                {
                    self.buffer = BytesMut::from(b);
                    if let Ok((p, _)) = P::deserialized(&p) {
                        return Ok(Some(p));
                    }
                }
            } else if let Ok((p, b)) = P::deserialized(&self.buffer) {
                self.buffer = BytesMut::from(b);
                return Ok(Some(p));
            }

            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .map_err(|e| e.to_string())?
            {
                return Err("Connection reset by peer".into());
            }
        }
    }
}

impl<P: Packet> ConnectionWriter<P> {
    pub async fn write_packet(
        &mut self,
        packet: P,
        secret: &Option<Vec<u8>>,
        nonce_generator: Option<&mut ChaCha20Rng>,
    ) -> std::io::Result<()> {
        let secret_and_nonce = if let Some(secret) = secret {
            let mut buf = [0u8; crate::SECRET_LEN];
            buf.copy_from_slice(&secret[..]);
            let mut nonce = [0u8; crate::NONCE_LEN];
            nonce_generator
                .expect("Expected `nonce_generator` to be `Some` because `secret` was `Some`.")
                .fill_bytes(&mut nonce);
            Some((buf, nonce))
        } else {
            None
        };
        let mut p = packet.serialized();
        if let Some((secret, nonce)) = secret_and_nonce {
            p = encrypt_frame(&p, &secret, &nonce);
        }
        self.stream.write_all(&p).await?;
        self.stream.flush().await
    }
}

mod encryption {
    use chacha20poly1305::{
        aead::{Aead, NewAead},
        XChaCha20Poly1305,
    };

    use crate::{NONCE_LEN, SECRET_LEN};

    pub fn encrypt_frame(
        packet_bytes: &[u8],
        key: &[u8; SECRET_LEN],
        nonce: &[u8; NONCE_LEN],
    ) -> Vec<u8> {
        // This maybe could use some unsafe pointer magic to be more optimal?
        let cipher = XChaCha20Poly1305::new(key.into());
        let len: u32 = packet_bytes.len().try_into().expect("Packet too big!");
        let mut buf = vec![0; len as usize + 4];
        buf[0..4].copy_from_slice(&len.to_be_bytes());
        debug_assert_eq!(buf[4..].len(), len as usize);
        let mut buf = cipher.encrypt(nonce.into(), packet_bytes).unwrap();
        let mut ret = vec![0u8; 4];
        let len: u32 = buf.len().try_into().expect("Packet too big!");
        ret.copy_from_slice(&len.to_be_bytes());
        ret.append(&mut buf);
        ret
    }

    pub fn decrypt_frame<'a>(
        encrypted_bytes: &mut &'a [u8],
        key: &[u8; SECRET_LEN],
        nonce: &[u8; NONCE_LEN],
    ) -> Result<(Vec<u8>, &'a [u8]), String> {
        if encrypted_bytes.len() < 4 {
            return Err("Too short".to_string());
        }

        let data_len: u32 = super::read_be_u32(encrypted_bytes);
        if data_len as usize > encrypted_bytes.len() {
            return Err("Not full frame".to_string());
        }

        let cipher = XChaCha20Poly1305::new(key.into());
        let (packet_bytes, rest) = encrypted_bytes.split_at(data_len as usize);
        let ret = cipher.decrypt(nonce.into(), packet_bytes).unwrap();
        Ok((ret, rest))
    }
}

fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}