# WhiteNoise Client
This is a client implementation in rust of WhiteNoise Protocol.

## WhiteNoise Network
WhiteNoise is an overlay privacy network protocol. 
It is designed to provide comprehensive network privacy protection,
including link privacy, node privacy, data privacy and traffic privacy. 

WhiteNoise Network is a decentralized network composed of nodes running the white noise protocol.

Learn more about the [WhiteNoise Protocol](https://github.com/Evanesco-Labs/WhiteNoise.rs).

## WhiteNoise Client
WhiteNoise Clients are able to access WhiteNoise network and build P2P privacy connection with other clients.
It is a multi-hop connection and several nodes in the WhiteNoise network ack as relays to transfer connection data.

Every client starts with an account associate with a crypto keypair. An unique WhiteNoiseID is derived from this keypair and
it identifies the client in WhiteNoise network. With privacy protection, dialing another client doesn't needs her IP
address, but only her WhiteNoiseID. So, clients who want to start connections share their WhiteNoiseIDs.

After Circuit Connection successfully built, both clients are able to read and write on this connection with network
privacy and security protection.

### Init Client
A client can be inited with the following method of `WhiteNoiseClient`:
```rust
pub fn init(bootstrap_addr_str: String, key_type: KeyType, keypair: Option<libp2p::identity::Keypair>) -> Self
```
Flag `bootstrap_addr_str` determines the Bootstrap node of the WhiteNoise network that the client is connected to. A
WhiteNoise network may have multiple Bootstraps, and only clients in the same WhiteNoise Network are able to connect to
each other.

Two kinds of key types Ed25519 and Secp256k1 are supported. If `keypair` is `None`, it will generate and store a new
keypair of the selected key type.


### Client APIs
Trait `Client` defines the APIs in the following:

- Get nodes of the WhiteNoise network. It return `PeerID` of no more than `cnt` nodes. PeerID is the identity of a
WhiteNoise node.

```rust
async fn get_main_net_peers(&mut self, cnt: i32) -> Vec<PeerId>
```


- Register to a node with `peer_id` as proxy to access the WhiteNoise network. Clients are able to chose proxy randomly or
set their own strategy.

```rust
async fn register(&mut self, peer_id: PeerId) -> bool
```


- Dial another client, and returns a SessionID if dialing success. SessionID is the unique identity of a circuit
connection, which shares by both sides of the connection. We supports multiplexing connection, so a client is able to
maintain multiple circuit connections. These connections are identified by SessionID.

```rust
async fn dial(&mut self, remote_id: String) -> String;
```


- Get circuit connection of certain SessionId.

```rust
fn get_circuit(&self, session_id: &str) -> Option<CircuitConn>
```


- Send message in a circuit connection of SessionID.

```rust
async fn send_message(&self, session_id: &str, data: &[u8])
```


- Close the circuit connection with session_id. This function will close the sub-streams of all relay nodes and the
clients of this circuit connection.

```rust
 async fn disconnect_circuit(&mut self, session_id: String)
```


7. Get client's unique WhiteNoiseId.

```rust
get_whitenoise_id( & self ) -> String
```


8. Pop latest inbound or outbound circuit connection's SessionID.

```rust
async fn notify_next_session(&mut self) -> Option<String>
```

## Build
Building WhiteNoise requires Rust toolchain. See more for how to install
Rust [here](https://www.rust-lang.org/tools/install).

Use the following command to build the WhiteNoise node:

```shell
cargo build --release
```

## Chat Example

We implement a P2P chatting application on WhiteNoise network as a demo. 

Copy `./target/release/whitenoise-client` into two different directories one as caller and another as answer. 
Try as follows:

First start local WhiteNoise Network or get the Bootstrap **MultiAddress** of a remote WhiteNoise network. More instructions [here](https://github.com/Evanesco-Labs/WhiteNoise.rs#start-local-whitenoise-network).

Start an chat **Answer** waiting for others to dial with this command, add your nick name in the `--nick` flag:

```shell
./whitenoise-client chat -b /ip4/127.0.0.1/tcp/3331/p2p/12D3KooWMNFaCGrnfMomi4TTMvQsKMGVwoxQzHo6P49ue6Fwq6zU --nick Alice
```

Your unique **WhiteNoiseID** is shown in log, this is your "number" for calls. **WhiteNoiseID** keeps the same, if you start chat example in the same directory and using the same key type.

The following shows the WhiteNoiseID in log:
```verilog
[2021-06-07T07:59:21.443Z INFO  whitenoisers::network::node] local whitenoise id:0HejBsyG9SPV5YB91Xf2zXiNGJQagRL3yAq7qtCVum4Pw
```

Start a chat **Caller** and dial the **Answer** with this command, fill in the `-n` flag with *Answer*'s *WhiteNoiseID*:

```shell
./whitenoise-client chat -b /ip4/127.0.0.1/tcp/3331/p2p/12D3KooWMNFaCGrnfMomi4TTMvQsKMGVwoxQzHo6P49ue6Fwq6zU --nick Bob -n 0HejBsyG9SPV5YB91Xf2zXiNGJQagRL3yAq7qtCVum4Pw
```

After seeing "Build circuit success!" in log, both chat clients are able to type chatting on the command line!

## Testing
To run basic test use:
```shell
cargo test
```