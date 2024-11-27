use std::{collections::HashMap, io};

use nom::{
    branch::alt,
    bytes::{
        complete::tag,
        streaming::{take_until, take_until1},
    },
    character::complete::crlf,
    combinator::value,
    error::ParseError,
    multi::many0,
    sequence::tuple,
    IResult,
};

const BUFFER_SIZE: usize = 32;
const CLRF: &str = "\r\n";

#[derive(Debug, Clone, PartialEq)]
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
    IOError(io::Error),
    NomError(String),
    Other(String),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::NomError(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
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

#[derive(Debug, PartialEq)]
struct Route(pub String);

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    method: Method,
    route: Route,
    headers: Headers,
    body: String,
}

type BufferNomError<'a> = nom::error::Error<&'a [u8]>;
type BufferNomResult<'a> = (&'a [u8], &'a [u8]);

pub struct HttpStream<S>
where
    S: io::Read,
{
    stream: S,
    rx: [u8; BUFFER_SIZE],
    received: Vec<u8>,
    stream_ended: bool,
}

impl<S> HttpStream<S>
where
    S: io::Read,
{
    pub fn new(stream: S) -> Self {
        HttpStream {
            stream,
            rx: [0u8; BUFFER_SIZE],
            received: Vec::with_capacity(1024),
            stream_ended: false,
        }
    }

    pub fn parse_request_socket(&mut self) -> MyResult<HttpRequest> {
        let (method, route) = self.parse_request_line()?;
        let headers = self.parse_headers()?;
        let body = self.parse_body(&headers)?;
        Ok(HttpRequest {
            method,
            route,
            headers,
            body,
        })
    }

    fn load_next_chunk(&mut self) -> MyResult<()> {
        if self.stream_ended {
            // A lil bit of a hack -- needs to be refactored
            return Ok(());
        }
        let bytes_read = self.stream.read(&mut self.rx)?;
        println!("Read {} bytes...", bytes_read);
        if bytes_read < BUFFER_SIZE {
            self.stream_ended = true;
        }
        self.received.extend_from_slice(&self.rx[..bytes_read]);
        Ok(())
    }

    fn fetch_next_line(&mut self) -> MyResult<Vec<u8>> {
        let mut result = Vec::new();
        loop {
            self.load_next_chunk()?;

            let line_raw =
                take_until::<_, _, BufferNomError>(CLRF)(self.received.as_slice());
            if is_complete(&line_raw) {
                let (_, parsed) = line_raw?;
                result.extend_from_slice(parsed);
                break;
            }
        }
        self.received.drain(..result.len());

        let (_, parsed) = crlf(self.received.as_slice())?;
        self.received.drain(..parsed.len());

        Ok(result)
    }

    fn parse_request_line(&mut self) -> MyResult<(Method, Route)> {
        let parsed = self.fetch_next_line()?;
        let (_, (method, _, route)) =
            tuple((parse_method, tag(" "), parse_route))(parsed.as_slice())?;

        Ok((method, route))
    }

    fn parse_headers(&mut self) -> MyResult<Headers> {
        let mut headers = HashMap::new();
        loop {
            let header_line = self.fetch_next_line()?;
            if header_line == "".as_bytes() {
                break;
            }

            let (_, (key, val)) = parse_header(header_line.as_slice())?;
            headers.insert(key, val);
        }
        Ok(headers)
    }

    fn parse_body(&mut self, headers: &Headers) -> MyResult<String> {
        let mut body = String::new();
        if let Some(expected_len) = headers.get("Content-Length") {
            let expected_len = expected_len.parse::<usize>().unwrap();
            while self.received.len() < expected_len {
                self.load_next_chunk()?;
            }
            body = String::from_utf8_lossy(self.received.as_slice()).into_owned();
        }
        Ok(body)
    }
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

fn parse_header(input: &[u8]) -> IResult<&[u8], (String, String)> {
    let (input, key) = take_until1(":")(input)?;
    let (input, _) = tag(":")(input)?;
    let (value, _) = many0(tag(" "))(input)?;
    let key = String::from_utf8_lossy(key).into_owned();
    let val = String::from_utf8_lossy(value).into_owned();
    Ok((input, (key, val)))
}

fn is_complete(input: &Result<BufferNomResult, nom::Err<BufferNomError>>) -> bool {
    if let Err(e) = input {
        return match e {
            nom::Err::Incomplete(_) => false,
            _ => true,
        };
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_request() {
        let request_raw = "\
            POST /wtfrouting HTTP/1.1\r\n\
            Content-Length: 20\r\n\
            Host: localhost:3333\r\n\
            \r\n\
            {\
                \"field\": \"example\"\
            }"
        .as_bytes();

        let mut stream = HttpStream::new(io::Cursor::new(request_raw));

        let parsed_request = stream.parse_request_socket().unwrap();
        let expected_request = HttpRequest {
            method: Method::POST,
            route: Route("/wtfrouting".to_owned()),
            headers: HashMap::from([
                ("Content-Length".into(), "20".into()),
                ("Host".into(), "localhost:3333".into()),
            ]),
            body: "{\"field\": \"example\"}".into(),
        };
        assert_eq!(parsed_request, expected_request);
    }
}
