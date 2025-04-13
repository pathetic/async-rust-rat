use common::commands::EncryptionRequestData;
use rsa::pkcs8::DecodePublicKey;
use rsa::Pkcs1v15Encrypt;
use std::net::TcpStream;

use common::commands::{ Command, EncryptionResponseData };
use common::buffers::write_buffer;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use rsa::RsaPublicKey;

pub fn generate_secret(write_stream: &mut TcpStream, data: EncryptionRequestData) {
    let padding = Pkcs1v15Encrypt::default();
    let public_key = RsaPublicKey::from_public_key_der(&data.public_key).unwrap();

    let secret = crate::SECRET.lock().unwrap();

    let encryption_response = EncryptionResponseData {
        secret: public_key
            .encrypt(&mut ChaCha20Rng::from_seed(*secret), padding, &*secret)
            .unwrap(),
    };

    *crate::SECRET_INITIALIZED.lock().unwrap() = true;
    write_buffer(write_stream, Command::EncryptionResponse(encryption_response), &None);
}
