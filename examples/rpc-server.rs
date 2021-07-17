use whitenoise_rpc::server::ServerBuilder;
use jsonrpc_core::{MetaIoHandler, Value};
use std::env::args;
use clap::{App, Arg};
use whitenoise_rpc::logger;

#[async_std::main]
async fn main() {
    logger::init_log();
    let args = App::new("dummy-server")
        .arg(Arg::with_name("bootstrap")
            .long("bootstrap")
            .short("b")
            .help("WhiteNoise network bootstrap node MultiAddress")
            .takes_value(true))
        .get_matches();

    let bootstrap_addr = args.value_of("bootstrap").unwrap();

    let (_tx, rx) = futures::channel::oneshot::channel::<()>();
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let mut io = MetaIoHandler::<()>::default();
    io.add_method("say_hello", |_params| Ok(Value::String("hello".to_string())));
    let mut builder = ServerBuilder::new(bootstrap_addr, Some(keypair), io);
    let _server = async_std::task::spawn(async move {
        builder.start().await
    }).await;
    rx.await;
}