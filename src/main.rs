use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    // based on https://www.youtube.com/watch?v=4DqP57BHaXI&ab_channel=ManningPublications
    // creating the echo server, that will receive messages and send than back
    // TODO: take care of the unwrap
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    // in order to send messages to all the users we need a broadcast channel
    // this `broadcast::channel` is multiple sender and receiver
    let (broadcast_sender, _broadcast_receiver) = broadcast::channel(10);
    loop {
        // TODO: take care of the unwrap
        let (mut socket, addr) = listener.accept().await.unwrap();
        let broadcast_sender = broadcast_sender.clone();
        let mut broadcast_receiver = broadcast_sender.subscribe();
        tokio::spawn(async move {
            //split the socket into read and write so we dont lose ownership of it
            let (read_half, mut write_half) = socket.split();
            // one kbyte buffer to get the messages
            // let mut buffer = [0u8; 1024];
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        // This point is after we receive the read_line future so we have result value
                        if result.unwrap() == 0 {
                            break;
                        }
                        broadcast_sender.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = broadcast_receiver.recv() => {
                        let (message, other_addr) = result.unwrap();
                        if addr != other_addr {
                            write_half.write_all(message.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
