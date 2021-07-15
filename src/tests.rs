use futures::{TryStreamExt, FutureExt, StreamExt};
use jsonrpc_core::{MetaIoHandler, Value};
use jsonrpc_core::futures::Future;
use futures::task::SpawnExt;
use std::time::Duration;
use env_logger::Builder;
use log::{info};
use crate::server::{ServerBuilder, start_test_server};
use crate::{DEFAULT_KEY_TYPE};
use whitenoisers::sdk::client::Client;
use libp2p::bytes::BufMut;

pub const LOCAL_BOOTSTRAP_ADDRESS: &str = "/ip4/127.0.0.1/tcp/6661/p2p/12D3KooWMNFaCGrnfMomi4TTMvQsKMGVwoxQzHo6P49ue6Fwq6zU";

#[async_std::test]
async fn single_request_test() {
    crate::logger::init_log();
    let server = start_test_server(LOCAL_BOOTSTRAP_ADDRESS).await;
    let response = async_std::task::spawn(send_request(server.unwrap().whitenosie_id())).await;
    info!("get response: {}", response);
    let response_expect = "{\"jsonrpc\":\"2.0\",\"result\":\"hello\",\"id\":1}";
    assert_eq!(response, response_expect)
}

async fn send_request(whitenoise_id: String) -> String {
    let keypair = libp2p::core::identity::Keypair::generate_ed25519();
    let mut client = whitenoisers::sdk::client::WhiteNoiseClient::init(
        LOCAL_BOOTSTRAP_ADDRESS.to_string(),
        whitenoisers::account::key_types::KeyType::from_text_str(DEFAULT_KEY_TYPE),
        Some(keypair));

    let peer_list = client.get_main_net_peers(10).await;
    let mut index = rand::random::<usize>();
    index %= peer_list.len();
    let proxy_remote_id = peer_list.get(index).unwrap();
    client.register(*proxy_remote_id).await;

    let session_id = client.dial(whitenoise_id).await;
    async_std::task::sleep(Duration::from_secs(1)).await;

    let mut circuit = client.get_circuit(session_id.as_str()).unwrap();
    let session_id = client.notify_next_session().await.unwrap();
    info!("{}", session_id.clone());
    info!("{:?}", circuit.transport_state.is_none());

    let data: Vec<u8> = b"{\"jsonrpc\": \"2.0\", \"method\": \"say_hello\", \"params\": [42, 23], \"id\": 1}\n"[..].to_owned();
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

#[async_std::test]
async fn test_stream() {
    let env = env_logger::Env::new().filter_or("MY_LOG", "info");
    let mut builder = Builder::new();
    builder.parse_env(env);
    builder.format_timestamp_millis();
    builder.init();

    let (_s, mut stop) = futures::channel::oneshot::channel::<()>();
    let mut io = MetaIoHandler::<()>::default();
    io.add_method("say_hello", |_params| Ok(Value::String("hello".to_string())));

    let (tx, rx) = futures::channel::mpsc::unbounded::<&str>();
    let (sender, receiver) = futures::channel::mpsc::unbounded::<Option<String>>();
    let response = rx.for_each(move |req| {
        info!("handle req");
        let res = io.handle_request(req, ()).wait().unwrap();
        sender.unbounded_send(res);
        futures::future::ready(())
    });

    async_std::task::spawn(response);

    async_std::task::spawn(receiver.for_each(|res| {
        info!("response {:?}", res);
        futures::future::ready(())
    }));

    info!("send");
    tx.unbounded_send("{\"jsonrpc\": \"2.0\", \"method\": \"say_hello\", \"params\": [42, 23], \"id\": 1}\n").unwrap();
    async_std::task::sleep(Duration::from_secs(1)).await;
    tx.unbounded_send("{\"jsonrpc\": \"2.0\", \"method\": \"say_hello\", \"params\": [42, 23], \"id\": 1}\n").unwrap();
    async_std::task::sleep(Duration::from_secs(1)).await;
    tx.unbounded_send("{\"jsonrpc\": \"2.0\", \"method\": \"say_hello\", \"params\": [42, 23], \"id\": 1}\n").unwrap();
    async_std::task::sleep(Duration::from_secs(1)).await;
    stop.await;
}

