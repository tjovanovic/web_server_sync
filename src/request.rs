use core::fmt;
use std::{collections::HashMap, io::Read, iter::repeat_with};

use nom::{
    branch::alt,
    bytes::{
        complete::tag,
        streaming::{take_until, take_until1},
    },
    character::complete::crlf,
    combinator::value,
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::tuple,
    IResult,
};

use crate::request;

const BUFFER_SIZE: usize = 32;
const CLRF: &str = "\r\n";

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

type Headers = HashMap<String, String>;

pub type MyResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    NomError(String),
    Other(String),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::NomError(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_: I, kind: nom::error::ErrorKind) -> Self {
        // let error = nom::error::Error::new(input, kind);
        Self::NomError(kind.description().into())
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug)]
struct Route(pub String);

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    route: Route,
    headers: Headers,
    body: String,
}

type BufferNomError<'a> = nom::error::Error<&'a [u8]>;
type BufferNomResult<'a> = (&'a [u8], &'a [u8]);

fn is_complete(g: &Result<BufferNomResult, nom::Err<BufferNomError>>) -> bool {
    match g {
        Ok(_) => true,
        Err(e) => match e {
            nom::Err::Incomplete(_) => false,
            _ => true,
        },
    }
}

fn preview_bytes(input: &[u8]) {
    print!("Preview > ");
    for x in input {
        print!("{}", *x as char);
    }
    println!();
}

pub fn parse_request_socket(stream: &mut impl Read) -> MyResult<HttpRequest> {
    let mut rx = [0u8; BUFFER_SIZE];
    let mut received = Vec::with_capacity(1024);

    let bytes_read = stream.read(&mut rx)?;
    received.extend_from_slice(&rx[..bytes_read]);
    let mut request_line = take_until1::<_, _, BufferNomError>(CLRF)(received.as_slice());

    while !is_complete(&request_line) {
        let bytes_read = stream.read(&mut rx)?;
        received.extend_from_slice(&rx[..bytes_read]);
        request_line = take_until1::<_, _, BufferNomError>(CLRF)(received.as_slice());
    }

    let (_, parsed) = request_line?;
    preview_bytes(parsed);
    let (_, (route, method)) = parse_request_line(&parsed)?;
    received.drain(..parsed.len());

    let (_, parsed) = crlf(received.as_slice())?;
    received.drain(..parsed.len());

    println!("Parsed: {:?}, {:?}", route, method);

    let mut headers = HashMap::new();
    let bytes_read = stream.read(&mut rx)?;
    received.extend_from_slice(&rx[..bytes_read]);

    loop {
        let mut header_line = take_until::<_, _, BufferNomError>(CLRF)(received.as_slice());
        while !is_complete(&header_line) {
            let bytes_read = stream.read(&mut rx)?;
            received.extend_from_slice(&rx[..bytes_read]);
            header_line = take_until::<_, _, BufferNomError>(CLRF)(received.as_slice());
        }

        let (_, header_line) = header_line?;

        if header_line == "".as_bytes() {
            println!("GG!!");
            break;
        }

        let (_, (key, val)) = parse_header(header_line)?;
        headers.insert(
            String::from_utf8_lossy(key).into_owned(),
            String::from_utf8_lossy(val).into_owned(),
        );
        received.drain(..header_line.len());

        let (_, parsed) = crlf(received.as_slice())?;
        received.drain(..parsed.len());
    }
    println!("Headers: {:?}", headers);

    let (_, parsed) = crlf(received.as_slice())?;
    received.drain(..parsed.len());

    let mut body = String::new();
    if let Some(l) = headers.get("Content-Length") {
        let l = l.parse::<usize>().unwrap();
        while received.len() < l {
            let bytes_read = stream.read(&mut rx)?;
            received.extend_from_slice(&rx[..bytes_read]);
        }
        body = String::from_utf8_lossy(received.as_slice()).into_owned();
    }

    let request = HttpRequest {
        method,
        route,
        headers,
        body,
    };
    Ok(request)
}

fn parse_request_line(input: &[u8]) -> IResult<&[u8], (Route, Method)> {
    let (input, (method, _, route)) = tuple((parse_method, tag(" "), parse_route))(input)?;

    Ok((input, (route, method)))
}

fn parse_route(input: &[u8]) -> IResult<&[u8], Route> {
    let (input, parsed) = take_until1(" ")(input)?;
    let s = String::from_utf8_lossy(parsed).into_owned();
    let route = Route(s);
    Ok((input, route))
}

fn parse_method(input: &[u8]) -> IResult<&[u8], Method> {
    alt((
        value(Method::GET, tag("GET")),
        value(Method::POST, tag("POST")),
        value(Method::PUT, tag("PUT")),
        value(Method::DELETE, tag("DELETE")),
    ))(input)
}

// fn preview_line(line: &str) -> String {
//     line.chars()
//         .into_iter()
//         .map(|c| match c {
//             '\r' => "\\r".to_owned(),
//             '\n' => "\\n".to_owned(),
//             c => c.to_string(),
//         })
//         .collect()
// }

// fn parse_headers(input: &str) -> IResult<&str, Headers> {
//     let (input, mut headers_raw) =
//         separated_list0(crlf, take_until("\r\n"))(input)?;

//     headers_raw.pop();

//     let mut headers = HashMap::new();
//     for header_raw in headers_raw {
//         let (_, (key, val)) = parse_header(header_raw)?;
//         headers.insert(key.to_owned(), val.to_owned());
//     }

//     Ok((input, headers))
// }

fn parse_header(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    let (input, key) = take_until1(":")(input)?;
    let (input, _) = tag(":")(input)?;
    let (value, _) = many0(tag(" "))(input)?;
    Ok((input, (key, value)))
}
