use std::net::UdpSocket;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use rand::Rng;
use tokio::sync::Mutex;
use tokio::time::sleep;

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
    pub last_heartbeat: Instant,
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
            last_heartbeat: Instant::now(),
            votes_received: 0,
        }
    }

    pub fn reset_election_timeout(&mut self) {
        self.election_timeout = rand::thread_rng().gen_range(150..300);
        self.last_heartbeat = Instant::now();
    }

    pub fn send_message(&self, socket: &UdpSocket, message: Message, target: &str) {
        let serialized = serde_json::to_vec(&message).unwrap();
        let _ = socket.send_to(&serialized, target);
        println!(
            "Node {} sent {:?} to {}",
            self.address, message, target
        );
    }

    pub fn handle_message(&mut self, message: Message, socket: &UdpSocket) {
        match message {
            Message::RequestVote { term, candidate_id } => {
                if term > self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    self.reset_election_timeout();
                    println!(
                        "Node {} voted for {} in term {}",
                        self.address, candidate_id, term
                    );
                    let response = Message::VoteGranted { term };
                    self.send_message(socket, response, &candidate_id);
                }
            }
            Message::AppendEntries { term, leader_id } => {
                if term >= self.current_term {
                    self.current_term = term;
                    self.state = NodeState::Follower;
                    self.reset_election_timeout();
                    println!(
                        "Node {} received heartbeat from Leader {} in term {}",
                        self.address, leader_id, term
                    );
                }
            }
            Message::VoteGranted { term } => {
                if self.state == NodeState::Candidate && term == self.current_term {
                    self.votes_received += 1;
                    println!(
                        "Node {} received a vote in term {}, total votes: {}",
                        self.address, term, self.votes_received
                    );
                    if self.votes_received > self.peers.len() / 2 {
                        self.state = NodeState::Leader;
                        println!("Node {} became Leader for term {}", self.address, self.current_term);
                    }
                }
            }
        }
    }
}

pub async fn start_node(node: Arc<Mutex<RaftNode>>) {
    let address = node.lock().await.address.clone();
    let socket = UdpSocket::bind(&address).expect("Failed to bind socket");
    socket.set_nonblocking(true).expect("Failed to set non-blocking");

    let mut buffer = [0; 1024];

    loop {
        {
            let mut node = node.lock().await;

            // Timeout handling for elections
            if node.last_heartbeat.elapsed() > Duration::from_millis(node.election_timeout as u64) {
                if node.state != NodeState::Leader {
                    node.state = NodeState::Candidate;
                    node.current_term += 1;
                    node.votes_received = 1; // Vote for itself
                    println!(
                        "Node {} started election for term {}",
                        node.address, node.current_term
                    );

                    let request = Message::RequestVote {
                        term: node.current_term,
                        candidate_id: node.address.clone(),
                    };

                    for peer in &node.peers {
                        node.send_message(&socket, request.clone(), peer);
                    }
                    node.reset_election_timeout();
                }
            }

            // Leader sends heartbeats
            if node.state == NodeState::Leader {
                let heartbeat = Message::AppendEntries {
                    term: node.current_term,
                    leader_id: node.address.clone(),
                };
                for peer in &node.peers {
                    node.send_message(&socket, heartbeat.clone(), peer);
                }
                println!("Leader {} sent heartbeats", node.address);
                sleep(Duration::from_millis(100)).await; // Heartbeat interval
            }
        }

        // Receive and handle incoming messages
        if let Ok((size, src)) = socket.recv_from(&mut buffer) {
            let message: Message = serde_json::from_slice(&buffer[..size]).unwrap();
            println!("Node {} received {:?} from {}", node.lock().await.address, message, src);
            let mut node = node.lock().await;
            node.handle_message(message, &socket);
        }

        std::thread::sleep(Duration::from_millis(10)); // Avoid busy looping
    }
}
