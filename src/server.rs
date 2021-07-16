use async_std::sync::Arc;
use jsonrpc_core::{MetaIoHandler, middleware, Middleware, Metadata};
use whitenoisers::account::key_types::KeyType;
use whitenoisers::sdk::client::{Client};
use std::io::Error;


use futures::{StreamExt};

use jsonrpc_core::futures::Future;
use futures::future;
use libp2p::bytes::BufMut;
use log::{info};
use crate::meta::{NoopExtractor, MetaExtractor, RequestContext};

const MAXREQUESTSIZE: usize = 1024;
const MAXRESPONSESIZE: usize = 1024;

pub struct Server {
    bootstrap_addr: String,
    whitenoise_id: String,
    stop: futures::channel::oneshot::Sender<()>,
}

pub struct ServerBuilder<M: Metadata = (), S: Middleware<M> = middleware::Noop> {
    handler: Arc<MetaIoHandler<M, S>>,
    bootstrap_addr: String,
    keypair: Option<libp2p::identity::Keypair>,
}

impl<M: Metadata + Default, S: Middleware<M>> ServerBuilder<M, S> {
    pub fn new<T>(bootstrap_addr: &str, keypair: Option<libp2p::identity::Keypair>, handler: T) -> ServerBuilder<M, S>
        where
            T: Into<MetaIoHandler<M, S>>,
    {
        ServerBuilder {
            handler: Arc::new(handler.into()),
            bootstrap_addr: bootstrap_addr.to_string(),
            keypair,
        }
    }

    //todo: over stack
    pub async fn start(&mut self) -> std::io::Result<Server> {
        //todo: stop
        let (stop_sender, _stop_rec) = futures::channel::oneshot::channel::<()>();

        let rpc_handler = self.handler.clone();
        let mut client = whitenoisers::sdk::client::WhiteNoiseClient::init(self.bootstrap_addr.clone(), KeyType::ED25519, self.keypair.clone());
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

                    let extractor = NoopExtractor {};
                    let meta = extractor.extract(&RequestContext {
                        session_id: circuit.id.clone(),
                    });
                    let response = handler.handle_request(&request, meta).wait().unwrap_or_else(|_|Some("rpc handle err".to_string()));
                    let response_bytes = response.unwrap_or_else(||"response none".to_string()).as_bytes().to_vec();
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
            whitenoise_id,
            stop: stop_sender,
        })
    }
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

    pub fn stop(self) {
        if !self.stop.is_canceled() {
            self.stop.send(()).unwrap();
        }
    }
}