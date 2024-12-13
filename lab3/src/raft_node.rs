use std::net::UdpSocket;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;

#[derive(Debug, PartialEq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    RequestVote { term: u64, candidate_id: String },
    AppendEntries { term: u64, leader_id: String },
    VoteGranted { term: u64 },
}

pub struct RaftNode {
    pub address: String,
    pub peers: Vec<String>,
    pub state: NodeState,
    pub current_term: u64,
    pub election_timeout: u64,
    pub votes_received: usize,
}

impl RaftNode {
    pub fn new(address: String, peers: Vec<String>) -> Self {
        RaftNode {
            address,
            peers,
            state: NodeState::Follower,
            current_term: 0,
            election_timeout: rand::thread_rng().gen_range(150..300),
            votes_received: 0,
        }
    }

    pub async fn handle_message(&mut self, msg: Message, socket: &UdpSocket) {
        match msg {
            Message::RequestVote { term, candidate_id } => {
                if term > self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    println!("Node {} voted for {}", self.address, candidate_id);
                    let response = Message::VoteGranted { term };
                    let serialized = serde_json::to_vec(&response).unwrap();
                    let _ = socket.send_to(&serialized, &candidate_id);
                }
            }
            Message::AppendEntries { term, leader_id } => {
                if term >= self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    println!("Node {} received heartbeat from Leader {}", self.address, leader_id);
                    self.reset_election_timeout();
                }
            }
            Message::VoteGranted { term } => {
                if term == self.current_term && self.state == NodeState::Candidate {
                    self.votes_received += 1;
                    println!("Node {} received a vote, total votes: {}", self.address, self.votes_received);
                    if self.votes_received > self.peers.len() / 2 {
                        self.state = NodeState::Leader;
                        println!("Node {} became Leader", self.address);
                    }
                }
            }
        }
    }

    pub async fn send_heartbeat(&self, socket: &UdpSocket) {
        if self.state == NodeState::Leader {
            let message = Message::AppendEntries {
                term: self.current_term,
                leader_id: self.address.clone(),
            };
            let serialized = serde_json::to_vec(&message).unwrap();
            for peer in &self.peers {
                let _ = socket.send_to(&serialized, peer);
            }
            println!("Leader {} sent heartbeats", self.address);
        }
    }

    pub fn reset_election_timeout(&mut self) {
        self.election_timeout = rand::thread_rng().gen_range(150..300);
    }
}

pub async fn start_node(node: Arc<Mutex<RaftNode>>) {
    let address = node.lock().await.address.clone();
    let socket = UdpSocket::bind(&address).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut buffer = [0; 1024];

    loop {
        {
            let mut node = node.lock().await;

            if node.state == NodeState::Follower || node.state == NodeState::Candidate {
                sleep(Duration::from_millis(node.election_timeout as u64)).await;
                if node.state != NodeState::Leader {
                    node.state = NodeState::Candidate;
                    node.current_term += 1;
                    node.votes_received = 1; // Vote for itself
                    println!("Node {} became Candidate for term {}", node.address, node.current_term);

                    let request = Message::RequestVote {
                        term: node.current_term,
                        candidate_id: node.address.clone(),
                    };

                    let serialized = serde_json::to_vec(&request).unwrap();
                    for peer in &node.peers {
                        let _ = socket.send_to(&serialized, peer);
                    }
                }
            }

            if node.state == NodeState::Leader {
                node.send_heartbeat(&socket).await;
                sleep(Duration::from_millis(100)).await; // Heartbeat interval
            }
        }

        if let Ok((size, _)) = socket.recv_from(&mut buffer) {
            let message: Message = serde_json::from_slice(&buffer[..size]).unwrap();
            let mut node = node.lock().await;
            node.handle_message(message, &socket).await;
        }
    }
}
