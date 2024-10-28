#![feature(thread_id_value)]

use std::{
    io::{BufRead, BufReader, Result as IOResult, Write},
    net::{TcpListener, TcpStream},
};

pub mod request;

use request::{parse_method, HttpRequest};

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

fn handle_client(mut stream: TcpStream) -> IOResult<()> {
    let thread_id = std::thread::current().id().as_u64();
    println!(
        "stream connected -- thread id {} -- process id {}",
        thread_id,
        std::process::id()
    );
    // stream.write(b"Stream connected!\n")?;
    let mut reader = BufReader::new(stream.try_clone()?);

    loop {
        // stream.write(b"> ")?;

        let mut line = String::new();

        let line_size = reader.read_line(&mut line)?;
        preview_line(&line, line_size);

        let (input, request) = parse_method(&line).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "ajnksdhfakjhfgkjsagd")
        })?;
        println!("Request: {:?}", request);

        if line == "\r\n" {
            println!("breaking!");
            break;
        }
    }

    // let response = format!("< {line}");

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

fn main() -> IOResult<()> {
    let listener = TcpListener::bind("127.0.0.1:3333")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let g = handle_client(stream?);
        match g {
            Ok(_p) => println!("All good..."),
            Err(e) => {
                println!("{:?} -- {:?}", e, e.raw_os_error());
            }
        }
        // std::thread::spawn(move || -> IOResult<()> {});
    }
    Ok(())
}
