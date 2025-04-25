use crate::commands::ClientCommand;
use common::packets::ClientboundPacket;
use common::ENC_TOK_LEN;
use rand::rngs::OsRng;
use rand::Rng;
use rsa::{PaddingScheme, RsaPrivateKey};
use tokio::sync::{mpsc::Sender, oneshot};

pub async fn handle_encryption_request(
    tx: Sender<ClientCommand>,
    otx: oneshot::Sender<Vec<u8>>,
    pub_key: Vec<u8>,
) {
    let mut token = [0u8; ENC_TOK_LEN];
    OsRng.fill(&mut token);
    tx.send(ClientCommand::Write(ClientboundPacket::EncryptionResponse(
        pub_key,
        token.to_vec(),
    )))
    .await
    .unwrap();
    otx.send(token.to_vec()).unwrap();
}

pub async fn handle_encryption_confirm(
    tx: Sender<ClientCommand>,
    otx: oneshot::Sender<Result<Vec<u8>, ()>>,
    enc_s: Vec<u8>,
    enc_t: Vec<u8>,
    exp_t: Vec<u8>,
    priv_key: RsaPrivateKey,
) {
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let t: Vec<u8> = priv_key
        .decrypt(padding, &enc_t)
        .expect("Failed to decrypt.");

    if t != exp_t {
        eprintln!("Encryption handshake failed!");
        tx.send(ClientCommand::Close).await.ok();
        otx.send(Err(())).unwrap();
    } else {
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let s = priv_key
            .decrypt(padding, &enc_s)
            .expect("Failed to decrypt.");
        otx.send(Ok(s.clone())).unwrap();
        tx.send(ClientCommand::SetSecret(Some(s.clone())))
            .await
            .unwrap();
        tx.send(ClientCommand::Write(ClientboundPacket::EncryptionAck))
            .await
            .unwrap();
        tx.send(ClientCommand::Write(ClientboundPacket::InitClient))
            .await
            .unwrap();
    }
}
