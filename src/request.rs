use core::fmt;
use std::{collections::HashMap, io::Read, iter::repeat_with};

use nom::{
    branch::alt,
    bytes::streaming::take_until1,
    character::complete::crlf,
    combinator::value,
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::tuple,
    IResult,
};

use crate::request;

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
    let mut buffer = vec![0; 10];
    let g = stream.read(&mut buffer)?;
    println!("Read {g} bytes...");

    let mut request_line =
        take_until1::<_, _, BufferNomError>("\r\n")(&buffer[..]);

    while !is_complete(&request_line) {
        let g = stream.read(&mut buffer)?;
        println!("Read {g} bytes...");
        request_line = take_until1::<_, _, BufferNomError>("\r\n")(&buffer[..]);
    }

    let (remaining, parsed) = request_line?;

    preview_bytes(parsed);
    preview_bytes(remaining);

    buffer.clear();
    buffer.extend_from_slice(remaining);

    Err(Error::Other("GG".into()))

    // let (_, (route, method)) = parse_request_line(&request_line)?;

    // let mut headers_raw = String::new();
    // loop {
    //     line.clear();
    //     reader.read_line(&mut line)?;
    //     headers_raw = format!("{headers_raw}{line}");
    //     println!("> Line: {}", line);

    //     if line == "\r\n" {
    //         println!("body now...!");
    //         break;
    //     }
    // }
    // println!("Headers raw:\n{}", headers_raw);

    // let (_, headers) = parse_headers(&headers_raw)?;

    // let mut body = String::new();
    // if let Some(l) = headers.get("Content-Length") {
    //     let l = l.parse::<usize>().unwrap();
    //     let mut buf = vec![0; l];
    //     reader.read_exact(&mut buf)?;

    //     body = buf.into_iter().map(|c| c as char).collect();
    // }

    // let request = HttpRequest {
    //     method,
    //     route,
    //     headers,
    //     body,
    // };
    // Ok(request)
}

// fn parse_request_line(input: &str) -> IResult<&str, (Route, Method)> {
//     let (input, (method, _, route)) =
//         tuple((parse_method, tag(" "), parse_route))(input)?;

//     Ok((input, (route, method)))
// }

// fn parse_route(input: &str) -> IResult<&str, Route> {
//     let (input, parsed) = take_until1(" ")(input)?;
//     let route = Route(parsed.to_owned());
//     Ok((input, route))
// }

// fn parse_method(input: &str) -> IResult<&str, Method> {
//     alt((
//         value(Method::GET, tag("GET")),
//         value(Method::POST, tag("POST")),
//         value(Method::PUT, tag("PUT")),
//         value(Method::DELETE, tag("DELETE")),
//     ))(input)
// }

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

// fn parse_header(input: &str) -> IResult<&str, (&str, &str)> {
//     println!("To parse: {}", preview_line(input));
//     let (input, key) = take_until1(":")(input)?;
//     let (input, _) = tag(":")(input)?;
//     let (remainder, _) = many0(tag(" "))(input)?;
//     println!("Remainder: {}", remainder);
//     // let (input, value) = take_until1("\r\n")(input)?;

//     Ok((input, (key, remainder)))
// }
