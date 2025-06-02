use chrono::{Duration, NaiveDate, NaiveDateTime};
use nom::{bytes::complete::take_while_m_n, combinator::map, IResult, Parser};

fn to_u32(s: &str) -> u32 {
    s.parse::<u32>().unwrap()
}

pub fn to_date(s: &str) -> Option<NaiveDateTime> {
    if s == "FFFF" {
        return None;
    }

    let days_to_add = i64::from_str_radix(s, 16).unwrap();

    let start_date = NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    Some(start_date + Duration::days(days_to_add))
}

fn is_dec_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn two_digit(input: &str) -> IResult<&str, u32> {
    map(take_while_m_n(2, 2, is_dec_digit), to_u32).parse(input)
}

pub fn four_alphanum(input: &str) -> IResult<&str, &str> {
    take_while_m_n(4, 4, |c: char| c.is_ascii_alphanumeric())(input)
}

pub fn two_alphanum(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_alphanumeric())(input)
}

pub fn date(input: &str) -> IResult<&str, NaiveDateTime> {
    map(four_alphanum, to_date)
        .parse(input)
        .map(|(i, d)| (i, d.unwrap()))
}

pub fn date_option(input: &str) -> IResult<&str, Option<NaiveDateTime>> {
    map(four_alphanum, to_date).parse(input)
}

// pub type BoxedParser<'a> = Box<dyn Parser<&'a str, &'a str, Error<&'a str>> + 'a>;
//
pub type BoxedParser<'a> = Box<dyn FnMut(&'a str) -> IResult<&'a str, &'a str> + 'a>;

// pub fn union_of_legit_symbol<'a>(min: usize, max: usize) -> impl FnMut(&'a str) -> Result<(&'a str, &'a str), nom::Err<nom::error::Error<&'a str>>> {
pub fn union_of_legit_symbol<'a>(min: usize, max: usize) -> BoxedParser<'a> {
    Box::new(take_while_m_n(min, max, |c: char| {
        c.is_ascii_alphanumeric() || c == '/' || c == ' ' || c == ','
    }))
}
