use std::{
    io::{ErrorKind, Read, Write},
    net::TcpListener,
    sync::mpsc::{channel, TryRecvError},
    thread,
    time::Duration,
};

const LOCAL_ADDR: &str = "127.0.0.1:3422";
const MESSAGE_SIZE: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL_ADDR).expect("listener failed to bind to local address");
    server
        .set_nonblocking(true)
        .expect("server failed to set to non blocking");

    let mut clients = vec![];

    let (sender, receiver) = channel::<String>();

    loop {
        match server.accept() {
            Ok((mut _socket, addr)) => {
                println!("Client {} connected", addr);
                let client = _socket.try_clone().expect("failed to clone socket");
                let sender = sender.clone();

                clients.push(client);

                thread::spawn(move || loop {
                    let mut message_buff = vec![0; MESSAGE_SIZE];
                    match _socket.read_exact(&mut message_buff) {
                        Ok(_) => {
                            let message =
                                String::from_utf8(message_buff).expect("buffer not valid utf8");
                            println!("client {} sent message {}", addr, message);
                            sender
                                .send(message)
                                .expect("failed to send message over channel");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("closing {}'s connection", addr);
                            break;
                        }
                    }
                });
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(e) => {
                println!("couldn't get client: {e:?}");
                break;
            }
        }

        match receiver.try_recv() {
            Ok(message) => {
                clients = clients
                    .into_iter()
                    .filter_map(|mut client| {
                        let mut message_buff = message.clone().into_bytes();
                        message_buff.resize(MESSAGE_SIZE, 0);
                        client.write_all(&message_buff).map(|_| client).ok()
                    })
                    .collect::<Vec<_>>();
            }
            Err(TryRecvError::Disconnected) => {
                println!("channel disconnected");
                break;
            }
            Err(_) => (),
        }

        thread::sleep(Duration::from_millis(100));
    }
}
