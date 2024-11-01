#![feature(thread_id_value)]
#![feature(buf_read_has_data_left)]

use std::{
    io::{BufRead, BufReader, Read, Result as IOResult, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
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
    stream.set_read_timeout(Some(Duration::new(5, 0)))?;
    // stream.write(b"Stream connected!\n")?;

    let mut reader = BufReader::new(stream.try_clone()?);

    let request = parse_request(reader);

    // let mut buf = Vec::new();

    // let mut s = String::new();
    // stream.read_to_string(&mut s)?;
    // println!("Stream: {}", s);

    let mut headers = String::new();
    loop {
        // stream.write(b"> ")?;

        let mut line = String::new();
        let line_size = reader.read_line(&mut line)?;
        headers = format!("{headers}{line}");
        preview_line(&line, line_size);

        // let (input, request) = parse_method(&line).map_err(|e| {
        //     std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        // })?;
        // println!("Request: {:?}", request);

        if line == "\r\n" {
            println!("body now...!");
            break;
        }
    }

    println!("Headers: {}", headers);

    // let x = reader.has_data_left()?;
    // println!("Data left: {}", x);

    // let x = reader.fill_buf()?;

    let mut buf = vec![0; 85];
    reader.read_exact(&mut buf)?;

    let s: String = buf.into_iter().map(|c| c as char).collect();
    println!("Body: {}", s);

    // for b in buf {
    //     println!("Char: {} -- {:x}", b as char, b as i32);
    // }

    // let s: String = x.into_iter().collect();

    // for b in x {
    //     println!("Char: {} -- {:x}", *b as char, *b as i32);
    // }

    // let mut buffer = Vec::with_capacity(15);

    // reader.read_exact(&mut buffer)?;
    // for b in buffer {
    //     println!("Char: {b} -- {:x}", b as i32);
    // }

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
