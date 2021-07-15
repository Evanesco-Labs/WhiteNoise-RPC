use async_std::sync::Arc;
use jsonrpc_core::{MetaIoHandler, middleware, Middleware, Metadata, Value};
use whitenoisers::account::key_types::KeyType;
use whitenoisers::sdk::client::{Client, WhiteNoiseClient};
use std::io::Error;
use futures::task::{Context, Poll};
use std::pin::Pin;
use futures::{Stream, StreamExt, AsyncRead, AsyncWrite};
use whitenoisers::network::connection::CircuitConn;
use jsonrpc_core::futures::Future;
use futures::future;
use libp2p::bytes::BufMut;
use log::{info};

const MAXREQUESTSIZE: usize = 1024;
const MAXRESPONSESIZE: usize = 1024;

pub struct Server {
    bootstrap_addr: String,
    keypair: libp2p::identity::Keypair,
    whitenoise_id: String,
    stop: futures::channel::oneshot::Sender<()>,
}

pub struct ServerBuilder {
    handler: Arc<MetaIoHandler<(), middleware::Noop>>,
    bootstrap_addr: String,
    keypair: libp2p::identity::Keypair,
}

impl ServerBuilder {
    pub fn new<T>(bootstrap_addr: &str, keypair: libp2p::identity::Keypair, handler: T) -> Self
        where
            T: Into<MetaIoHandler<(), middleware::Noop>>,
    {
        ServerBuilder {
            handler: Arc::new(handler.into()),
            bootstrap_addr: bootstrap_addr.to_string(),
            keypair,
        }
    }

    pub async fn start(&mut self) -> std::io::Result<Server> {
        let (stop_sender, stop_rec) = futures::channel::oneshot::channel::<()>();

        let rpc_handler = self.handler.clone();
        let mut client = whitenoisers::sdk::client::WhiteNoiseClient::init(self.bootstrap_addr.clone(), KeyType::ED25519, Some(self.keypair.clone()));
        let handler = rpc_handler.clone();

        let peer_list = client.get_main_net_peers(10).await;
        if peer_list.is_empty() {
            return Err(Error::new(std::io::ErrorKind::Other, "bootstrap has no peers"));
        }
        let mut index = rand::random::<usize>();
        index %= peer_list.len();
        let proxy_remote_id = peer_list.get(index).unwrap();
        info!("choose id:{:?} to register", proxy_remote_id);
        client.register(*proxy_remote_id).await;
        let node = client.node.clone();
        let whitenoise_id = client.get_whitenoise_id();


        //init service handler
        let service = client.new_connected_session.map(move |session_id| {
            node.circuit_map.read().unwrap().get(session_id.as_str()).cloned()
        }).filter_map(|circuit_option| async move {
            if circuit_option.is_some() {
                circuit_option
            } else {
                None
            }
        }).for_each(move |mut circuit| {
            let handler = handler.clone();
            async_std::task::spawn(async move {
                loop {
                    let mut buf = [0u8; MAXREQUESTSIZE];
                    let data = circuit.read(&mut buf).await;
                    if data.is_empty() {
                        info!("data empty");
                        break;
                    }
                    let request = String::from_utf8_lossy(&data).into_owned();
                    let response = handler.handle_request(&request, ()).wait().unwrap_or(Some({ "rpc handle err".to_string() }));
                    let response_bytes = response.unwrap_or("response none".to_string()).as_bytes().to_vec();
                    let payload = wrap_message(&response_bytes);
                    let mut buf = [0u8; MAXRESPONSESIZE];
                    circuit.write(payload.as_slice(), &mut buf).await;
                }
            });
            future::ready(())
        });


        async_std::task::spawn(service);


        Ok(Server {
            bootstrap_addr: self.bootstrap_addr.clone(),
            keypair: self.keypair.clone(),
            whitenoise_id,
            stop: stop_sender,
        })
    }
}

pub async fn start_test_server(bootstrap_addr: &str) -> std::io::Result<Server> {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let mut io = test_handler();
    let mut builder = ServerBuilder::new(bootstrap_addr, keypair, io);
    async_std::task::spawn(async move {
        builder.start().await
    }).await
}

pub fn test_handler() -> MetaIoHandler<()> {
    let mut io = MetaIoHandler::<()>::default();
    io.add_method("say_hello", |_params| Ok(Value::String("hello".to_string())));
    io
}

pub fn wrap_message(message: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(4 + message.len());
    payload.put_u32(message.len() as u32);
    payload.chunk_mut().copy_from_slice(message);
    unsafe {
        payload.advance_mut(message.len());
    }
    payload
}

impl Server {
    pub fn whitenosie_id(&self) -> String {
        self.whitenoise_id.clone()
    }
}

//
// pub struct CircuitStream(pub WhiteNoiseClient);
//
// impl futures::Stream for CircuitStream {
//     type Item = CircuitConn;
//
//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         unimplemented!()
//     }
// }

//
// pub struct WhiteNoiseSocket(pub CircuitConn);
//
// impl AsyncRead for WhiteNoiseSocket {
//     fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
//         unimplemented!()
//     }
// }
//
// impl AsyncWrite for WhiteNoiseSocket {
//     fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
//         unimplemented!()
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
//         unimplemented!()
//     }
//
//     fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
//         unimplemented!()
//     }
// }
