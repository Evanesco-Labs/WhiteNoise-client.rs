use libp2p::PeerId;
use whitenoisers::network::connection::CircuitConn;
use async_trait::async_trait;
pub mod node;
pub mod client;

#[async_trait]
pub trait Client {
    ///Get nodes of the WhiteNoise network.
    async fn get_main_net_peers(&mut self, cnt: i32) -> Vec<PeerId>;
    ///Register to a node as proxy to access the WhiteNoise network.
    async fn register(&mut self, peer_id: PeerId) -> bool;
    ///Dial another client, and returns a unique session id if dialing success.
    async fn dial(&mut self, remote_id: String) -> String;
    ///Get circuit connection of certain session id.
    fn get_circuit(&self, session_id: &str) -> Option<CircuitConn>;
    ///Send message in a circuit connection with session_id.
    async fn send_message(&self, session_id: &str, data: &[u8]);
    ///Close the circuit connection with session_id.
    async fn disconnect_circuit(&mut self, session_id: String);
    ///Get client's unique WhiteNoise Id.
    fn get_whitenoise_id(&self) -> String;
    ///Pop latest inbound or outbound circuit connection's session id.
    async fn notify_next_session(&mut self) -> Option<String>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
