#![feature(thread_id_value)]
#![feature(buf_read_has_data_left)]

use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

pub mod request;

use request::{parse_request_socket, MyResult};

fn main() -> MyResult<()> {
    let listener = TcpListener::bind("127.0.0.1:3333")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let g = handle_client(stream?);
        match g {
            Ok(_p) => println!("All good..."),
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        // std::thread::spawn(move || -> IOResult<()> {});
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> MyResult<()> {
    let thread_id = std::thread::current().id().as_u64();
    println!(
        "stream connected -- thread id {} -- process id {}",
        thread_id,
        std::process::id()
    );
    stream.set_read_timeout(Some(Duration::new(5, 0)))?;
    // stream.write(b"Stream connected!\n")?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let request = parse_request_socket(&mut reader)?;
    println!("Request: {:#?}", request);

    let response = format!("HTTP/1.1 200 OK\r\n");
    stream.write(&response.into_bytes())?;
    stream.write("Content-Length: 12\r\n".as_bytes())?;
    stream.write("Content-Type: text/html\r\n".as_bytes())?;
    stream.write("Connection: close\r\n".as_bytes())?;
    stream.write("\r\n".as_bytes())?;
    stream.write("Hello world!\r\n".as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn preview_line(line: &str, size: usize) {
    let line_prev: String = line
        .chars()
        .into_iter()
        .map(|c| match c {
            '\r' => "\\r".to_owned(),
            '\n' => "\\n".to_owned(),
            c => c.to_string(),
        })
        .collect();

    println!("> {line_prev} -- {size}",);
}

fn preview_chars(line: &str) {
    for c in line.chars() {
        if c == '\n' {
            println!("Char: \\n -- {:x}", c as i32);
        } else if c == '\r' {
            println!("Char: \\r -- {:x}", c as i32);
        } else {
            println!("Char: {c} -- {:x}", c as i32);
        }
    }
}
