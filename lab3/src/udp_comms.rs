use tokio::net::UdpSocket;

pub async fn send_message(socket: &UdpSocket, message: &[u8], address: &str) {
    let _ = socket.send_to(message, address).await;
}
