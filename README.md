# WhiteNoise-RPC

Implementation of jsonrpc server and client based on WhiteNoise Network transport.

## WhiteNoise Network

WhiteNoise is an overlay privacy network protocol. It is designed to provide comprehensive network privacy protection,
including link privacy, node privacy, data privacy and traffic privacy.

WhiteNoise Network is a decentralized network composed of nodes running the white noise protocol.

Learn more about the [WhiteNoise Protocol](https://github.com/Evanesco-Labs/WhiteNoise.rs).

## Build

Building WhiteNoise-RPC requires Rust toolchain. See more for how to install
Rust [here](https://www.rust-lang.org/tools/install).

Use the following command to build the WhiteNoise node:

```shell
cargo build --release
```

## RPC Server

WhiteNoise-RPC server is based on [paritytech/jsonrpc](https://github.com/paritytech/jsonrpc) with WhiteNoise Network
transport. It handle rpc requests with `IoHandler`. You can check
this [document](https://docs.rs/jsonrpc-core/17.1.0/jsonrpc_core/struct.IoHandler.html) for specifics
of `IoHandler`.

Before starting a WhiteNoise-RPC server or client, you need to know the Bootstrap MultiAddress of a WhiteNoise Network.
For testing, you can deploy local WhiteNoise Network and get the Bootstrap MultiAddress, follow this [instruction](https://github.com/Evanesco-Labs/WhiteNoise.rs#start-local-whitenoise-network).

Here is an [example](./examples/rpc-server.rs) of how to run a rpc server. Use this command to start the example server:

```shell
cargo run --example rpc-server -- -b <Bootstrap MultiAddress>
```

WhiteNoiseID of the server is shown in log. The WhiteNoiseID is `0GyqhYzYLepmueNg8wknmjtbqacJtZyDJNnMPtqt6uXT9` in this log example:

```
[2021-07-16T09:40:55Z INFO  whitenoisers::network::node] [WhiteNoise] local whitenoise id:0GyqhYzYLepmueNg8wknmjtbqacJtZyDJNnMPtqt6uXT9
```

## RPC Client

This executable client read json request from file and send this request to WhiteNoise-RPC server and print response.

### Send Request

First there is a WhiteNoise-RPC server starts up and connected to a WhiteNoise Network. Before sending request, you need to know the
Bootstrap MultiAddress of the WhiteNoise Network and the WhiteNoiseID of the server.

Then generate a new file and write json request to this file. For example copy the following text to ./request.json as a
test request:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "say_hello",
  "params": [
    42,
    23
  ]
}
```

Run the following command to send request and the response is shown in log:

```shell
./target/release/whitenoise-rpc --bootstrap <Bootstrap MultiAddress> --id <WhiteNoiseID> --json <json request file path>
```

If you use the test request above and send it to the example server, you can see response in log:

```shell
response: {"jsonrpc":"2.0","result":"hello","id":1}
```

Run this command to explore all parameters and commands:

```shell
./target/release/whitenoise-rpc -h
```

## Test
Tests including rpc requests to example rpc server and substrate WhiteNoise-based rpc server. 
So before start testing you have to first start a local WhiteNoise network, follow this [instruction](https://github.com/Evanesco-Labs/WhiteNoise.rs#start-local-whitenoise-network).
Remember the bootstrap node address shown in log.
For better description, we assume that the bootstrap node address we start is `/ip4/118.190.163.66/tcp/3331/p2p/12D3KooWJGJ1guC3JsVWWpx6N5acGEzpMZbFDbNpAnZkU6ADwwPA`.

Then startup a substrate node-template with WhiteNoise-based rpc on, follow this [instruction](https://github.com/Evanesco-Labs/substrate#start-substrate-node-template).
Notice that you should fill in the `--whitenoise-bootstrap` with the bootstrap node address you've just started. In this case, it's `/ip4/118.190.163.66/tcp/3331/p2p/12D3KooWJGJ1guC3JsVWWpx6N5acGEzpMZbFDbNpAnZkU6ADwwPA`.

Remember this WhiteNoiseID printed in your log. Here the WhiteNoiseID is `07sYJEC6MiSP6PZBuhq6KJUwgHhJNvwVWipySMR8peVJs` in the following log:

```shell
2021-06-28T05:39:36.426Z INFO  whitenoisers::network::node] [WhiteNoise] local whitenoise id:07sYJEC6MiSP6PZBuhq6KJUwgHhJNvwVWipySMR8peVJs
```

Change two const values in file `./src/tests.rs` to customize local test network:
1. Change `const TEST_BOOTSTRAP_ADDRESS` to your local bootstrap node address:
  
`const TEST_BOOTSTRAP_ADDRESS: &str = "/ip4/127.0.0.1/tcp/6661/p2p/12D3KooWMNFaCGrnfMomi4TTMvQsKMGVwoxQzHo6P49ue6Fwq6zU";`

2. Change `const TEST_SERVER_ID` to your substrate node-template WhiteNoiseID:

`const TEST_SERVER_ID: &str = "06ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt";`

Finally run this command to test:
`cargo test`