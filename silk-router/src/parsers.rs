//! Parsers for url segments

use std::iter::Peekable;
use std::str::Chars;

use num::{CheckedAdd, CheckedMul, Num, NumCast};

/// Parses a number from a URL
pub fn num<T>(iter: &mut Peekable<Chars>) -> Option<T>
where
    T: Num + NumCast + CheckedAdd + CheckedMul,
{
    let mut consumed = 0u8;
    let mut result = T::zero();
    for ch in iter {
        consumed += 1;
        let x = match ch.to_digit(10) {
            Some(x) => <T as NumCast>::from(x).unwrap(),
            None => return None,
        };
        result = match result.checked_mul(&<T as NumCast>::from(10).unwrap()) {
            Some(result) => result,
            None => return None,
        };
        result = match result.checked_add(&x) {
            Some(result) => result,
            None => return None,
        };
    }
    if consumed == 0 {
        None
    } else {
        Some(result)
    }
}

/// Parses a string up to a given delimiter character
pub fn until(delim: char) -> impl FnMut(&mut Peekable<Chars>) -> Option<String> {
    move |peekable| {
        let mut result = String::new();
        for ch in peekable {
            if ch == delim {
                return Some(result);
            }
            result.push(ch);
        }
        None
    }
}

// Parses the entire rest of a URL
pub fn rest(iter: &mut Peekable<Chars>) -> Option<String> {
    Some(iter.collect())
}
