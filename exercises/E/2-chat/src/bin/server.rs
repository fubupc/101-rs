use std::sync::Arc;

use anyhow::Result;
use chat::Message;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::broadcast,
    task,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").await?;
    let (tx, _) = broadcast::channel(1024);
    let tx = Arc::new(tx);
    loop {
        let (stream, _) = tcp_listener.accept().await?;
        let (tcp_read, tcp_write) = stream.into_split();
        let peer_addr = match tcp_read.peer_addr() {
            Ok(peer_addr) => peer_addr,
            Err(err) => {
                eprintln!("cannot get peer address: {}", err);
                continue;
            }
        };
        println!("[peer@{peer_addr}] connection established");

        task::spawn({
            let tx = tx.clone();
            let peer_addr = peer_addr.clone();
            async move {
                match handle_incoming(tcp_read, tx).await {
                    Ok(_) => {}
                    Err(err) => eprintln!("[peer@{peer_addr}] ERROR: {err}"),
                }
            }
        });

        task::spawn({
            let rx = tx.subscribe();
            let peer_addr = peer_addr.clone();
            async move {
                match handle_outgoing(tcp_write, rx).await {
                    Ok(_) => {}
                    Err(err) => eprintln!("[peer@{peer_addr}] ERROR: {err}"),
                }
            }
        });
    }
}

async fn handle_incoming(
    tcp_read: OwnedReadHalf,
    tx: impl AsRef<broadcast::Sender<Message>>,
) -> Result<()> {
    let mut tcp_read = BufReader::new(tcp_read).lines();
    let Some(initial_message) =  tcp_read.next_line().await? else {
        return Err(anyhow::format_err!(
            "close connection without sending initial message"
        ));
    };

    // todo!(
    //     "Deserialize initial_message into a Message::User.
    //         If the initial line is not a Message::User, stop this task."
    // );
    let Message::User(mut user) = serde_json::from_str(&initial_message)? else {
        return Err(anyhow::format_err!("initial message is not Message:User: {initial_message}"));
    };
    println!("<{user}> joined chat");
    tx.as_ref().send(Message::User(user.clone()))?;

    // todo!("For each further incoming line, deserialize the line into a Message");
    // todo!("If the message is a Message::User, broadcast the message as-is using tx");
    // todo!(
    //     "If the message is a Message::SimpleMessage,
    //     convert it into a Message::Chat and broadcast it using tx"
    // );
    // todo!("If the message is a Message::Chat, ignore it");
    while let Some(line) = tcp_read.next_line().await? {
        let msg: Message = serde_json::from_str(&line)?;
        match msg {
            Message::User(new_user) => {
                // TODO: What to do when new_user is different than previous user?
                user = new_user; // overwrite previous user?
                tx.as_ref().send(Message::User(user.clone()))?;
            }
            Message::ClientMessage(content) => {
                tx.as_ref().send(Message::Chat {
                    user: user.clone(),
                    content,
                })?;
            }
            Message::Chat {
                user: _,
                content: _,
            } => {
                // Client should not send this kind of message, ignore
            }
        };
    }

    Ok(())
}

async fn handle_outgoing(
    mut tcp_write: OwnedWriteHalf,
    rx: broadcast::Receiver<Message>,
) -> Result<()> {
    let mut rx = BroadcastStream::from(rx);
    while let Some(msg) = rx.next().await.transpose()? {
        // todo!(
        //     "Serialize message as JSON and send it to the client,
        //     along with a newline"
        // );
        tcp_write
            .write_all((serde_json::to_string(&msg)? + "\n").as_bytes())
            .await?;
    }
    Ok(())
}
