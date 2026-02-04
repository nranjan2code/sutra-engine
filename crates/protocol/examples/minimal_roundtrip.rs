use std::error::Error;
use tokio::net::TcpListener;

use sutra_protocol::{recv_message, send_message, StorageMessage, StorageResponse};

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // Spawn a tiny echo-style server that understands StorageMessage.
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await?;
            let _msg: StorageMessage = recv_message(&mut socket).await?;
            send_message(
                &mut socket,
                &StorageResponse::HealthCheckOk {
                    healthy: true,
                    status: "ok".to_string(),
                    uptime_seconds: 1,
                },
            )
            .await?;
            Ok::<(), std::io::Error>(())
        });

        // Client side.
        let mut client = tokio::net::TcpStream::connect(addr).await?;
        send_message(&mut client, &StorageMessage::HealthCheck).await?;
        let resp: StorageResponse = recv_message(&mut client).await?;

        println!("Received response: {:?}", resp);
        server.await??;
        Ok::<(), Box<dyn Error>>(())
    })?;

    Ok(())
}
