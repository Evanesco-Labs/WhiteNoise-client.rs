use libp2p::{Multiaddr,
             PeerId, Transport,
             core::{upgrade::{self}},
             identify, mplex, noise::{NoiseConfig, X25519Spec, Keypair},
             request_response::{ProtocolSupport, RequestResponseConfig, RequestResponse},
             swarm::{SwarmBuilder}, tcp::TcpConfig};
use whitenoisers::network::{protocols::cmd_protocol::{CmdCodec, CmdProtocol}, whitenoise_behaviour::{WhitenoiseClientBehaviour}};

use std::{iter};

use futures::channel::mpsc;
use log::{debug};
use whitenoisers::network::{whitenoise_behaviour::{WhitenoiseBehaviour}};
use whitenoisers::network::protocols::proxy_protocol::{ProxyCodec, ProxyProtocol};
use whitenoisers::network::protocols::ack_protocol::{AckCodec, AckProtocol};
use whitenoisers::network::protocols::relay_behaviour;

use whitenoisers::account::account::Account;

use whitenoisers::network::whitenoise_behaviour::{self};

use libp2p::kad::{
    Kademlia,
    KademliaConfig,
};
use libp2p::kad::record::store::MemoryStore;

use whitenoisers::network::node::Node;
use whitenoisers::network::proxy_event_handler::process_proxy_request;
use whitenoisers::network::cmd_event_handler::process_cmd_request;

pub fn new_client_node(bootstrap_addr_option: std::option::Option<String>, key_pair: Option<libp2p::identity::Keypair>, key_type: whitenoisers::account::key_types::KeyType) -> Node {
    let (node_request_sender, node_request_receiver) = mpsc::unbounded();

    let id_keys = match key_pair {
        None => Account::get_default_account_keypair("./db", key_type),
        Some(x) => x
    };

    let peer_id = id_keys.public().into_peer_id();
    debug!("local peer id: {:?}", peer_id);

    let noise_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
    let trans = TcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::default())
        .timeout(std::time::Duration::from_millis(150))
        .boxed();


    let proxy_protocols = iter::once((ProxyProtocol(), ProtocolSupport::Full));
    let proxy_cfg = RequestResponseConfig::default();
    let proxy_behaviour = RequestResponse::new(ProxyCodec(), proxy_protocols, proxy_cfg);

    let cmd_protocols = iter::once((CmdProtocol(), ProtocolSupport::Full));
    let cmd_cfg = RequestResponseConfig::default();
    let cmd_behaviour = RequestResponse::new(CmdCodec(), cmd_protocols, cmd_cfg);

    let ack_protocols = iter::once((AckProtocol(), ProtocolSupport::Full));
    let ack_cfg = RequestResponseConfig::default();
    let ack_behaviour = RequestResponse::new(AckCodec(), ack_protocols, ack_cfg);

    let identify_behaviour = identify::Identify::new(identify::IdentifyConfig::new(String::from("/ipfs/id/1.0.0"), id_keys.public()).with_initial_delay(std::time::Duration::from_millis(500)).with_interval(std::time::Duration::from_secs(5 * 60)));

    let relay_behaviour = relay_behaviour::Relay {
        out_events: std::collections::VecDeque::new(),
        addresses: std::collections::HashMap::new(),
        dive_events: std::collections::VecDeque::new(),
    };

    let event_bus = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));

    let relay_in_streams = std::sync::Arc::new(std::sync::RwLock::new(std::collections::VecDeque::new()));
    let relay_out_streams = std::sync::Arc::new(std::sync::RwLock::new(std::collections::VecDeque::new()));

    let mut dht_cfg = KademliaConfig::default();
    let whitnoise_dht_protocol: &[u8] = b"/whitenoise_dht/kad/1.0.0";
    dht_cfg.set_protocol_name(std::borrow::Cow::Borrowed(whitnoise_dht_protocol));
    dht_cfg.set_query_timeout(std::time::Duration::from_secs(5 * 60));
    let store = MemoryStore::new(peer_id);
    let mut kad_behaviour = Kademlia::with_config(peer_id, store, dht_cfg);
    match bootstrap_addr_option {
        None => {}
        Some(bootstrap_addr) => {
            let parts: Vec<&str> = bootstrap_addr.split('/').collect();
            let bootstrap_peer_id_str = parts.last().unwrap();
            let bootstrap_peer_id = PeerId::from_bytes(bs58::decode(bootstrap_peer_id_str).into_vec().unwrap().as_slice()).unwrap();
            let mut bootstrap_addr_multiaddr: Multiaddr = bootstrap_addr.parse().unwrap();
            let index_opt = bootstrap_addr.find("p2p");
            if let Some(index) = index_opt {
                let bootstrap_addr_parts = bootstrap_addr.split_at(index - 1);
                bootstrap_addr_multiaddr = bootstrap_addr_parts.0.parse().unwrap();
            }
            kad_behaviour.add_address(&bootstrap_peer_id, bootstrap_addr_multiaddr);
        }
    }

    let (proxy_request_sender, proxy_request_receiver) = mpsc::unbounded();
    let (cmd_request_sender, cmd_request_receiver) = mpsc::unbounded();
    let node = Node {
        node_request_sender,
        event_bus: event_bus.clone(),
        keypair: id_keys,
        proxy_id: None,
        relay_in_streams: relay_in_streams.clone(),
        relay_out_streams: relay_out_streams.clone(),
        circuit_task: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        client_peer_map: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        client_wn_map: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        session_map: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        circuit_map: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        probe_map: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
    };

    let whitenoise_behaviour = WhitenoiseBehaviour {
        proxy_behaviour,
        cmd_behaviour,
        ack_behaviour,
        event_bus,
        relay_in_streams,
        relay_out_streams,
        relay_behaviour,
        proxy_request_channel: proxy_request_sender,
        cmd_request_channel: cmd_request_sender,
    };
    let whitenoise_client_behaviour = WhitenoiseClientBehaviour {
        whitenoise_behaviour,
        identify_behaviour,
    };
    let swarm1 = SwarmBuilder::new(trans, whitenoise_client_behaviour, peer_id)
        .executor(Box::new(|fut| {
            async_std::task::spawn(fut);
        }))
        .build();

    async_std::task::spawn(whitenoise_behaviour::whitenoise_client_event_loop(swarm1, node_request_receiver));

    async_std::task::spawn(process_proxy_request(proxy_request_receiver, node.clone()));
    async_std::task::spawn(process_cmd_request(cmd_request_receiver, node.clone()));
    node
}