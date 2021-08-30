use std::{env, io, net::SocketAddr, sync::mpsc::{self, Receiver}, thread};
use std::net::UdpSocket;

#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr) => { dbg!($x) }
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($x:expr) => { std::convert::identity($x) }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "server"{
        let stdin_channel = start_server("0.0.0.0:34254");
        // let timeout = Duration::new(0, 1000000000);
        loop {
            match stdin_channel.try_recv() {
                Ok(elem) => {}, //println!("{:?}", elem),
                Err(_) => sleep(10),
            }
        }
    }else if args.len() > 1 {
        start_sender(&(args[1].as_str().to_owned() + ":34254"));
        //start_sender("127.0.0.1:34254")
    }
    // println!("Hello, world!");
}

fn sleep(millis: u64) {
    let sleep_time = std::time::Duration::from_millis(millis);
    std::thread::sleep(sleep_time);
}

fn create_udp_socket_receiver(host: &str) -> io::Result<UdpSocket> {
    // debug!("initializing listener udp socket on {}", host);
    let socket = UdpSocket::bind(&host)?;
    return Ok(socket);
}

fn create_udp_socket_sender(host: &str) -> io::Result<UdpSocket> {
    let local_address = "0.0.0.0:0";
    let socket = UdpSocket::bind(local_address)?;
    let socket_address: SocketAddr = host
        .parse::<SocketAddr>()
        .expect("Invalid forwarding address specified");
    socket.connect(&socket_address)?;
    // debug!("initializing listener udp socket on {}", local_address);
    return Ok(socket);
}

fn start_sender(host: &str) {
    let listen_socket = create_udp_socket_sender(host).expect("failed to bind host socket");
    for num in 0..1000000 {
        let buf = u32::to_le_bytes(num);
        let res = send(&listen_socket, host, &buf);
        if let Some(tmp_res) = res{
            let mut tmp = [0; 4];
            for i in 0..3 {
                tmp[i] = tmp_res[i];
            }
            let tmp_i = u32::from_le_bytes(tmp);
            if num != tmp_i{
                println!("{}, {}", num, tmp_i);
            }
        }
    }
}

fn send(socket: &UdpSocket, receiver: &str, msg: &[u8]) -> Option<Vec<u8>> {
    // println!("sending message: {:?}", msg);
    let result: usize = 0;
    match socket.send_to(&msg, receiver) {
        Ok(number_of_bytes) => {},//println!("{:?}", number_of_bytes),
        Err(fail) => println!("failed sending {:?}", fail),
    }
    let mut buf = [0; 20];
    return match socket.recv(&mut buf){
        Ok(number_of_bytes) => Option::Some(Vec::from(&buf[0..number_of_bytes])),
        Err(_) => Option::None,
    };
}

fn start_server(host: &str) -> Receiver<Vec<u8>>{
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let socket = create_udp_socket_receiver(host).unwrap();
    thread::spawn(move || {
        let mut buf = [0; 20];
        let mut result: Vec<u8>;
        loop{
            match socket.recv_from(&mut buf){
                Ok((number_of_bytes, src)) => {
                    result = Vec::from(&buf[0..number_of_bytes]);
                    // debug!(&result);
                    tx.send(result).unwrap_or_default();
                    if number_of_bytes > 0 {
                        socket.send_to(&buf, &src).unwrap_or_default();
                    }
                },
                Err(err) => print!("{:?}", err)
            }
        }
    });
    rx
}
