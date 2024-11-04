use std::{
    collections::HashMap,
    io::{BufRead, Result as IOResult},
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_until1},
    character::complete::crlf,
    combinator::value,
    multi::{many0, separated_list0},
    sequence::tuple,
    IResult,
};

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

type Headers = HashMap<String, String>;

#[derive(Debug)]
struct Route(pub String);

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    route: Route,
    headers: Headers,
    body: String,
}

pub fn parse_request_socket(
    reader: &mut impl BufRead,
) -> IOResult<HttpRequest> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let request_line = line.clone();

    let (_, (route, method)) =
        parse_request_line(&request_line).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;

    let mut headers_raw = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        headers_raw = format!("{headers_raw}{line}");
        println!("> Line: {}", line);

        if line == "\r\n" {
            println!("body now...!");
            break;
        }
    }
    println!("Headers raw:\n{}", headers_raw);

    let (_, headers) = parse_headers(&headers_raw).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    let mut body = String::new();
    if let Some(l) = headers.get("Content-Length") {
        let l = l.parse::<usize>().unwrap();
        let mut buf = vec![0; l];
        reader.read_exact(&mut buf)?;

        body = buf.into_iter().map(|c| c as char).collect();
    }

    let request = HttpRequest {
        method,
        route,
        headers,
        body,
    };
    Ok(request)
}

fn parse_request_line(input: &str) -> IResult<&str, (Route, Method)> {
    let (input, (method, _, route)) =
        tuple((parse_method, tag(" "), parse_route))(input)?;

    Ok((input, (route, method)))
}

fn parse_route(input: &str) -> IResult<&str, Route> {
    let (input, parsed) = take_until1(" ")(input)?;
    let route = Route(parsed.to_owned());
    Ok((input, route))
}

fn parse_method(input: &str) -> IResult<&str, Method> {
    alt((
        value(Method::GET, tag("GET")),
        value(Method::POST, tag("POST")),
        value(Method::PUT, tag("PUT")),
        value(Method::DELETE, tag("DELETE")),
    ))(input)
}

fn preview_line(line: &str) -> String {
    line.chars()
        .into_iter()
        .map(|c| match c {
            '\r' => "\\r".to_owned(),
            '\n' => "\\n".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

fn parse_headers(input: &str) -> IResult<&str, Headers> {
    let (input, mut headers_raw) =
        separated_list0(crlf, take_until("\r\n"))(input)?;

    headers_raw.pop();

    let mut headers = HashMap::new();
    for header_raw in headers_raw {
        let (_, (key, val)) = parse_header(header_raw)?;
        headers.insert(key.to_owned(), val.to_owned());
    }

    Ok((input, headers))
}

fn parse_header(input: &str) -> IResult<&str, (&str, &str)> {
    println!("To parse: {}", preview_line(input));
    let (input, key) = take_until1(":")(input)?;
    let (input, _) = tag(":")(input)?;
    let (remainder, _) = many0(tag(" "))(input)?;
    println!("Remainder: {}", remainder);
    // let (input, value) = take_until1("\r\n")(input)?;

    Ok((input, (key, remainder)))
}
