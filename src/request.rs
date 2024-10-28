use core::error;

use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{Error, ParseError};
use nom::IResult;
use serde::de::value::Error as SerdeError;
use serde::de::IntoDeserializer;
use serde::Deserialize;

// type IResult<T> = nom::IResult<T, T>;

#[derive(Deserialize, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}
#[derive(Deserialize, Debug)]
pub struct HttpRequest {
    method: Method,
    // host: String,
    // port: String,
}

fn map_errs<'a>(error: SerdeError) -> nom::Err<nom::error::Error<&'a str>> {
    // let g = nom::error::Error::new(&error.to_string()[..], nom::error::ErrorKind::Alpha);
    // let g = nom::error::Error::new("jebem ti mater u picku", nom::error::ErrorKind::Alpha);
    // let x = nom::Err::Error(g);
    let g: String = error.to_owned().to_string();
    let x = nom::Err::Error(nom::error::Error::new(
        "jebem ti kevu mrtvu",
        nom::error::ErrorKind::Alpha,
    ));
    x
}

pub fn parse_method(input: &str) -> IResult<&str, HttpRequest> {
    let (input, output) = tag("GET")(input)?;
    // let x = Method::deserialize(output.into_deserializer()).map_err(map_errs)?;
    // let x = Method::deserialize(output.into_deserializer()).map_err(map_errs);
    let x: Result<_, SerdeError> = Method::deserialize(output.into_deserializer());
    let method = x.map_err(map_errs)?;
    Ok((input, HttpRequest { method }))
}

// pub fn parse_headers(line: &str) -> IResult<&str> {}
