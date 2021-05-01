use std::{
    fs,
    io::prelude::*,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use std::{
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use learn_rust_web_server::ThreadPool;

fn main() {
    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr).unwrap();
    let pool = ThreadPool::new(4).unwrap();

    let flag = Arc::new(AtomicBool::new(false));
    let terminate_flag = flag.clone();

    ctrlc::set_handler(move || {
        println!("Terminating... press ctrl+c again to force exit");
        if !flag.load(Ordering::SeqCst) {
            flag.store(true, Ordering::SeqCst);
            // send nonsense connection in-case we're idling
            // alternative was to spin on the cpu in non-blocking mode :/
            let _ = TcpStream::connect(addr);
        } else {
            exit(1); //already trying trying to terminate, so lets hard exit
        }
    })
    .unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // println!("Connection established");
        pool.execute(|| {
            handle_connection(stream);
        });

        if terminate_flag.load(Ordering::SeqCst) {
            break;
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
