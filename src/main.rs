use whitenoise_rpc::client;
use clap::{App, Arg};

#[async_std::main]
async fn main() {
    // crate::logger::init_log();
    let args = App::new("whitenoise-rpc-cli")
        .version("1.0")
        .author("EVA-Labs")
        .about("whitenoise-rpc-cli")
        .arg(Arg::with_name("bootstrap")
            .long("bootstrap")
            .short("b")
            .help("WhiteNoise network bootstrap node MultiAddress")
            .takes_value(true))
        .arg(Arg::with_name("id")
            .long("id")
            .help("WhiteNoise ID of the WhiteNoise RPC server")
            .takes_value(true))
        .arg(Arg::with_name("json")
            .long("json")
            .help("json-rpc request path")
            .default_value("./request.json")
            .takes_value(true))
        .get_matches();

    let bootstrap_addr = args.value_of("bootstrap").unwrap();
    let whitenosie_id = args.value_of("id").unwrap();
    let json_path = args.value_of("json").unwrap();
    let request =  std::fs::read_to_string(json_path).unwrap();

    let response = async_std::task::block_on(client::send_request(bootstrap_addr, whitenosie_id.to_string(), request));

    println!("response: {}", response);
}

