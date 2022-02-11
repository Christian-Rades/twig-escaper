use std::{env::args, error, fmt::Display, fs::File, io::Read, io::Write};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    multi::many0,
    sequence::delimited,
    IResult,
};
use regex::Regex;

fn main() -> Result<(), Box<dyn error::Error>> {
    let path = args()
        .nth(1)
        .expect("first arg is the file path to a md file");
    let mut f = File::open(&path)?;
    let mut text = String::default();

    f.read_to_string(&mut text)?;
    drop(f);

    let (rest, result) = parse(&text).unwrap();

    let out = File::create(&path)?;

    let twig_regex = Regex::new("\\{%.*%\\}")?;

    for s in result {
        match s {
            DocumentPiece::Code(code) if twig_regex.is_match(code) => {
                writeln!(&out, "{{% raw %}}")?;
                write!(&out, "{}", s)?;
                writeln!(&out, "\n{{% endraw %}}")?;
            }
            _ => write!(&out, "{}", s)?,
        }
    }
    write!(&out, "{}", rest)?;

    Ok(())
}

#[derive(Debug)]
enum DocumentPiece<'a> {
    Text(&'a str),
    Code(&'a str),
}

impl Display for DocumentPiece<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentPiece::Text(text) => write!(f, "{}", text),
            DocumentPiece::Code(code) => write!(f, "```{}```", code),
        }
    }
}

fn parse(i: &str) -> IResult<&str, Vec<DocumentPiece<'_>>> {
    let (rest, mut results) = many0(alt((parse_code_piece, parse_text_piece)))(i)?;
    results.push(DocumentPiece::Text(rest));
    Ok(("", results))
}

fn parse_text_piece(i: &str) -> IResult<&str, DocumentPiece> {
    let (rest, text) = take_until("```")(i)?;
    Ok((rest, DocumentPiece::Text(text)))
}

fn parse_code_piece(i: &str) -> IResult<&str, DocumentPiece> {
    let (rest, code) = delimited(tag("```"), take_until("```"), tag("```"))(i)?;
    Ok((rest, DocumentPiece::Code(code)))
}
