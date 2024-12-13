mod raft_node;
mod udp_comms;

use tokio::task;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let nodes = vec!["127.0.0.1:8081", "127.0.0.1:8082", "127.0.0.1:8083"];

    let handles: Vec<_> = nodes.iter()
        .map(|&address| {
            let peers = nodes.iter().filter(|&&peer| peer != address).map(|&peer| peer.to_string()).collect::<Vec<_>>();
            let node = Arc::new(Mutex::new(raft_node::RaftNode::new(address.to_string(), peers)));
            task::spawn(raft_node::start_node(node))
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }
}
