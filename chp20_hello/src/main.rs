use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use chp20_hello::ThreadPool;

const HTML_200_PATH: &str = "hello.html";
const HTML_404_PATH: &str = "404.html";
const N_CALLS: usize = 3;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(N_CALLS) {
        let stream = stream.unwrap();

        pool.execute(|| handle_connection(stream));
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, html_path) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", HTML_200_PATH),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", HTML_200_PATH)
        }
        _ => ("HTTP/1.1 404 OK", HTML_404_PATH),
    };
    let content = fs::read_to_string(html_path).unwrap();
    let content_length = content.len();
    let response = format!(
        "{status_line}
        Content-Length: {content_length}\r\n\r\n
        {content}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}
