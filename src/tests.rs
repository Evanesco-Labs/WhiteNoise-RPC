use futures::{TryStreamExt, FutureExt, StreamExt};
use jsonrpc_core::{MetaIoHandler, Value};
use jsonrpc_core::futures::Future;
use futures::task::SpawnExt;
use std::time::Duration;
use env_logger::Builder;
use log::{info};
use crate::server::{ServerBuilder, start_test_server};
use crate::{DEFAULT_KEY_TYPE, client};
use whitenoisers::sdk::client::Client;
use libp2p::bytes::BufMut;
use crate::client::send_request;

const TEST_BOOTSTRAP_ADDRESS: &str = "/ip4/127.0.0.1/tcp/6661/p2p/12D3KooWMNFaCGrnfMomi4TTMvQsKMGVwoxQzHo6P49ue6Fwq6zU";
const TEST_SERVER_ID: &str = "05aMGHVxUaPerqgDSSxwWsS3G7cyJhHbJx9id6YfUMkLg";

#[async_std::test]
async fn single_request_test() {
    crate::logger::init_log();
    let server = start_test_server(TEST_BOOTSTRAP_ADDRESS).await;
    let request = "{\"jsonrpc\": \"2.0\", \"method\": \"say_hello\", \"params\": [42, 23], \"id\": 1}\n";
    let response = async_std::task::spawn(send_request(TEST_BOOTSTRAP_ADDRESS, server.unwrap().whitenosie_id(), request.to_string())).await;
    info!("get response: {}", response);
    let response_expect = "{\"jsonrpc\":\"2.0\",\"result\":\"hello\",\"id\":1}";
    assert_eq!(response, response_expect)
}

#[async_std::test]
async fn substrate_request_test() {
    crate::logger::init_log();
    let insert_key_request = "{
    \"jsonrpc\":\"2.0\",
     \"id\":1,
     \"method\":\"author_insertKey\",
     \"params\": [
    \"aura\",
    \"clip organ olive upper oak void inject side suit toilet stick narrow\",
    \"0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e\"
    ]\
    }";
    let response = async_std::task::block_on(client::send_request(TEST_BOOTSTRAP_ADDRESS, TEST_SERVER_ID.to_string(), insert_key_request.to_string()));
    let expect_res = "{\"jsonrpc\":\"2.0\",\"result\":null,\"id\":1}";
    info!("{}", response);
    assert_eq!(response, expect_res.to_string());

    async_std::task::sleep(Duration::from_secs(1));

    let has_key_request: &str = "{
  \"jsonrpc\":\"2.0\",
  \"id\":1,
  \"method\":\"author_hasKey\",
  \"params\": [
    \"0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e\",
    \"aura\"
  ]\
  }";
    let response = async_std::task::block_on(client::send_request(TEST_BOOTSTRAP_ADDRESS, TEST_SERVER_ID.to_string(), has_key_request.to_string()));
    let expect_response = "{\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}";
    info!("{}", response);
    assert_eq!(response, expect_response.to_string());
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

