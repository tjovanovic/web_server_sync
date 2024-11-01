use std::{collections::HashMap, str::FromStr};
use std::{
    io::{BufRead, BufReader, Read, Result as IOResult, Write},
    net::TcpStream,

use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

type Headers = HashMap<String, String>;

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    headers: Headers,
    body: String,
    // host: String,
    // port: String,
}

pub fn parse_request_socket(reader: &impl BufRead) {
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
}

pub fn parse_method(input: &str) -> IResult<&str, Method> {
    alt((
        value(Method::GET, tag("GET")),
        value(Method::POST, tag("POST")),
        value(Method::PUT, tag("PUT")),
        value(Method::DELETE, tag("DELETE")),
    ))(input)
}

// pub fn parse_headers(line: &str) -> IResult<&str> {}
