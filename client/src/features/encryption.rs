use std::sync::Mutex;
use once_cell::sync::Lazy;
use rand::{rngs::OsRng, Rng, SeedableRng};
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use rand_chacha::ChaCha20Rng;
use common::{connection::{Connection, ConnectionReader, ConnectionWriter}, packets::*};

pub static SECRET: Lazy<Mutex<[u8; common::SECRET_LEN]>> = Lazy::new(||
    Mutex::new([0u8; common::SECRET_LEN])
);

pub struct EncryptionState {
    pub secret: Option<Vec<u8>>,
    pub nonce_generator_read: Option<ChaCha20Rng>,
    pub nonce_generator_write: Option<ChaCha20Rng>,
}

impl EncryptionState {
    pub fn new() -> Self {
        Self {
            secret: None,
            nonce_generator_read: None,
            nonce_generator_write: None,
        }
    }

    pub fn initialize_with_secret(&mut self, secret_vec: Vec<u8>) {
        SECRET.lock().unwrap().copy_from_slice(&secret_vec[..]);
        let mut seed = [0u8; common::SECRET_LEN];
        seed.copy_from_slice(&secret_vec[..]);
        self.secret = Some(secret_vec);
        self.nonce_generator_read = Some(ChaCha20Rng::from_seed(seed));
        self.nonce_generator_write = Some(ChaCha20Rng::from_seed(seed));
    }
}

pub async fn perform_encryption_handshake(
    connection: Connection<ClientboundPacket, ServerboundPacket>
) -> Result<(
    EncryptionState, 
    ConnectionReader<ClientboundPacket>, 
    ConnectionWriter<ServerboundPacket>
), &'static str> {
    let (mut reader, mut writer) = connection.split();
    let mut enc_state = EncryptionState::new();

    writer
        .write_packet(
            ServerboundPacket::EncryptionRequest,
            &enc_state.secret,
            enc_state.nonce_generator_write.as_mut(),
        )
        .await
        .map_err(|_| "Failed to send encryption request")?;

    let (pub_key, token): (RsaPublicKey, Vec<u8>) = if let Ok(Some(p)) = reader
        .read_packet(&enc_state.secret, enc_state.nonce_generator_read.as_mut())
        .await
    {
        match p {
            ClientboundPacket::EncryptionResponse(pub_key_der, token_) => {
                let pub_key = rsa::pkcs8::FromPublicKey::from_public_key_der(&pub_key_der)
                    .map_err(|_| "Failed to parse server public key")?;
                assert_eq!(common::ENC_TOK_LEN, token_.len());
                (pub_key, token_)
            }
            _ => return Err("Unexpected packet during encryption handshake"),
        }
    } else {
        return Err("Failed to read encryption response");
    };

    let mut secret = [0u8; common::SECRET_LEN];
    OsRng.fill(&mut secret);

    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let enc_secret = pub_key
        .encrypt(&mut OsRng, padding, &secret[..])
        .map_err(|_| "Failed to encrypt secret")?;
    
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let enc_token = pub_key
        .encrypt(&mut OsRng, padding, &token[..])
        .map_err(|_| "Failed to encrypt token")?;
    
    writer
        .write_packet(
            ServerboundPacket::EncryptionConfirm(enc_secret, enc_token),
            &None,
            enc_state.nonce_generator_write.as_mut(),
        )
        .await
        .map_err(|_| "Failed to send encryption confirmation")?;

    enc_state.initialize_with_secret(secret.to_vec());

    match reader
        .read_packet(&enc_state.secret, enc_state.nonce_generator_read.as_mut())
        .await 
    {
        Ok(Some(ClientboundPacket::EncryptionAck)) => {
            // Return encryption state along with reader and writer
            Ok((enc_state, reader, writer))
        }
        Ok(_) => Err("Unexpected packet instead of encryption acknowledgment"),
        Err(_) => Err("Failed to read encryption acknowledgment"),
    }
}
