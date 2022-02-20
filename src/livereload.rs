use std::{io, net::TcpListener, sync::mpsc::Receiver};

use log::{error, info};
use rayon::spawn;
use tungstenite::{accept, Message};

pub fn start_reload_server(reload_rx: Receiver<()>) -> io::Result<()> {
    info!("Starting reload server");
    let server = TcpListener::bind("127.0.0.1:9001")?;
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            info!("new connection");

            let mut line = String::new();
            let input = std::io::stdin().read_line(&mut line);

            if let Err(error) = websocket.write_message(Message::Text("reload".to_string())) {
                error!("Failed to send ws message: {}", error)
            }
        });
    }

    Ok(())
}
