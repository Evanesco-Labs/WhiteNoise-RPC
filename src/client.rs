use crate::DEFAULT_KEY_TYPE;
use whitenoisers::sdk::client::Client;
use std::time::Duration;
use libp2p::bytes::BufMut;

pub async fn send_request(boostrap_addr: &str, whitenoise_id: String, request: String) -> String {
    let keypair = libp2p::core::identity::Keypair::generate_ed25519();
    let mut client = whitenoisers::sdk::client::WhiteNoiseClient::init(
        boostrap_addr.to_string(),
        whitenoisers::account::key_types::KeyType::from_text_str(DEFAULT_KEY_TYPE),
        Some(keypair));

    let peer_list = client.get_main_net_peers(10).await;
    let mut index = rand::random::<usize>();
    index %= peer_list.len();
    let proxy_remote_id = peer_list.get(index).unwrap();
    client.register(*proxy_remote_id).await;
    client.dial(whitenoise_id).await;
    let session_id = client.notify_next_session().await.unwrap();

    let mut circuit = client.get_circuit(session_id.as_str()).unwrap();
    info!("{}", session_id.clone());
    info!("{:?}", circuit.transport_state.is_none());

    let data: Vec<u8> = request.into_bytes();
    let message = data.as_slice();
    let mut payload = Vec::with_capacity(4 + message.len());
    payload.put_u32(message.len() as u32);
    payload.chunk_mut().copy_from_slice(message);
    unsafe {
        payload.advance_mut(message.len());
    }

    client.send_message(session_id.as_str(), &payload).await;
    info!("finish send request");

    let mut buf = [0u8; 1024];
    let data = circuit.read(&mut buf).await;
    let response = String::from_utf8_lossy(data.as_slice()).to_owned();
    response.to_string()
}