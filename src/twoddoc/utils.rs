use chrono::{Duration, NaiveDate};
use nom::{bytes::complete::take_while_m_n, combinator::map, error::Error, IResult, Parser};

fn to_u32(s: &str) -> u32 {
    s.parse::<u32>().unwrap()
}

pub fn to_date(s: &str) -> NaiveDate {
    let days_to_add = i64::from_str_radix(s, 16).unwrap();

    let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();

    start_date + Duration::days(days_to_add)
}

fn is_dec_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn two_digit(input: &str) -> IResult<&str, u32> {
    map(take_while_m_n(2, 2, is_dec_digit), to_u32)(input)
}

pub fn four_alphanum(input: &str) -> IResult<&str, &str> {
    take_while_m_n(4, 4, |c: char| c.is_ascii_alphanumeric())(input)
}

pub fn two_alphanum(input: &str) -> IResult<&str, &str> {
    take_while_m_n(2, 2, |c: char| c.is_ascii_alphanumeric())(input)
}

pub fn date(input: &str) -> IResult<&str, NaiveDate> {
    map(four_alphanum, to_date)(input)
}

pub type BoxedParser<'a> = Box<dyn Parser<&'a str, &'a str, Error<&'a str>> + 'a>;

pub fn alphanumeric<'a>(min: usize, max: usize) -> BoxedParser<'a> {
    Box::new(take_while_m_n(min, max, |c: char| {
        c.is_ascii_alphanumeric()
    }))
}

pub fn alphanumeric_space_slash<'a>(min: usize, max: usize) -> BoxedParser<'a> {
    Box::new(take_while_m_n(min, max, |c: char| {
        c.is_ascii_alphanumeric() || c == '/' || c == ' '
    }))
}
