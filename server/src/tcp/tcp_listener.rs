use crate::binary::sender::SenderKind;
use crate::streaming::clients::client_manager::Transport;
use crate::streaming::systems::system::SharedSystem;
use crate::tcp::connection_handler::{handle_connection, handle_error};
use std::net::SocketAddr;
use tokio::net::TcpSocket;
use tokio::sync::oneshot;
use tracing::{error, info};

pub async fn start(address: &str, socket: TcpSocket, system: SharedSystem) -> SocketAddr {
    let address = address.to_string();
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        let addr = address.parse();
        if addr.is_err() {
            panic!("Unable to parse address {:?}", address);
        }

        socket
            .bind(addr.unwrap())
            .expect("Unable to bind socket to address");

        let listener = socket.listen(1024).expect("Unable to start TCP server.");

        let local_addr = listener
            .local_addr()
            .expect("Failed to get local address for TCP listener");

        tx.send(local_addr).unwrap_or_else(|_| {
            panic!(
                "Failed to send the local address {:?} for TCP listener",
                local_addr
            )
        });

        loop {
            match listener.accept().await {
                Ok((stream, address)) => {
                    info!("Accepted new TCP connection: {address}");
                    let session = system
                        .read()
                        .await
                        .add_client(&address, Transport::Tcp)
                        .await;

                    let client_id = session.client_id;
                    info!("Created new session: {session}");
                    let system = system.clone();
                    let mut sender = SenderKind::get_tcp_sender(stream);
                    tokio::spawn(async move {
                        if let Err(error) =
                            handle_connection(session, &mut sender, system.clone()).await
                        {
                            handle_error(error);
                            system.read().await.delete_client(client_id).await;
                            if let Err(error) = sender.shutdown().await {
                                error!("Failed to shutdown TCP stream for client: {client_id}, address: {address}. {error}");
                            } else {
                                info!("Successfully closed TCP stream for client: {client_id}, address: {address}.");
                            }
                        }
                    });
                }
                Err(error) => error!("Unable to accept TCP socket. {error}"),
            }
        }
    });
    match rx.await {
        Ok(addr) => addr,
        Err(_) => panic!("Failed to get the local address for TCP listener."),
    }
}
