use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler, ActorContext};
use actix_web::{web, HttpRequest, HttpResponse};
use std::io::Write;
use std::net::TcpStream;
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Message)]
#[rtype(result = "()")]
struct BroadcastMessage(pub String);

// Chat Server to manage rooms and clients
pub struct ChatServer {
    rooms: Arc<Mutex<HashMap<String, Vec<Addr<Client>>>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// WebSocket Client Actor
pub struct Client {
    room: String,
    server: Arc<Mutex<HashMap<String, Vec<Addr<Client>>>>>,
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let mut rooms = self.server.lock().unwrap();
        rooms.entry(self.room.clone()).or_default().push(addr);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let mut rooms = self.server.lock().unwrap();
        if let Some(clients) = rooms.get_mut(&self.room) {
            clients.retain(|client_addr| client_addr != &addr);
        }
    }
}

// Handle incoming WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Client {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let message = format!("write {}", text); // Format as a TCP write command

                // Send to TCP server
                if let Ok(mut stream) = TcpStream::connect("127.0.0.1:9000") {
                    stream.write_all(message.as_bytes()).expect("Failed to send to TCP server");
                }

                // Broadcast to other clients in the room
                let mut rooms = self.server.lock().unwrap();
                if let Some(clients) = rooms.get_mut(&self.room) {
                    for client in clients.iter() {
                        let _ = client.do_send(BroadcastMessage(text.to_string()));
                    }
                }
            }
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => {}
        }
    }
}

// Handle broadcast messages
impl Handler<BroadcastMessage> for Client {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// WebSocket route handler
pub async fn start_chat(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<Mutex<HashMap<String, Vec<Addr<Client>>>>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let room = req.match_info().query("room").to_string();

    let server = data.get_ref().clone();
    let client = Client {
        room,
        server,
    };

    ws::start(client, &req, stream)
}
