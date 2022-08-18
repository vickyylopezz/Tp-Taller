use crate::bencode::bencoded_value::BencodedValue;
use crate::bencode::parser_error::ParserError;
use std::str;

/// # Parser
/// This type is tasked with parsing bencoded data
pub struct Parser {
    /// Characters in the bencoded string
    characters: Vec<u8>,
    /// Cursor that point to the current character
    cursor: usize,
}

impl Iterator for Parser {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl Parser {
    /// Parses the string if it's a valid bencoded byte string and
    /// returns a Result<BencodedValue, ParserError>. Advances the
    /// cursor.
    ///
    /// # Errors
    ///
    /// This function will return an error if there an error parsing
    /// the length of the byte string. It will only parse the number
    /// of characters specified by the length.
    ///
    /// # Examples
    ///
    /// ```#s
    /// let s = "6:example"
    /// let bencoded_byte_string = parse(s)?;
    ///
    /// assert_eq!(bencoded_byte_string, BencodedValue::ByteString("example"".into()));
    /// ```
    ///
    fn byte_string(&mut self) -> Result<BencodedValue, ParserError> {
        let length = self
            .take_while(|c| (*c as char).is_alphanumeric() && *c != b':')
            .map(|c| c as char)
            .collect::<String>()
            .parse()
            .map_err(|_| ParserError::InvalidByteStringLength)?;
        let byte_string = self.take(length).collect::<Vec<u8>>();

        Ok(BencodedValue::ByteString(byte_string))
    }
    /// Parses the string if it's a valid bencoded dictionary and
    /// returns a Result<BencodedValue, ParserError>. Advances the
    /// cursor.
    ///
    /// # Errors
    ///
    /// This function will return an error if there an error parsing
    /// one of the elements of the dictionary or if the the dictionary
    /// is missing the *e* delimiter indicating the end of the
    /// dictionary.
    ///
    /// # Examples
    ///
    /// ```#s
    /// let s = "d3:onei1e3:twoi2e5:threei3ee"
    /// let bencoded_dictionary = parse(s)?;
    /// let dict = vec![
    ///         (
    ///             BencodedValue::ByteString("one".into()),
    ///             BencodedValue::Integer(1),
    ///         ),
    ///         (
    ///             BencodedValue::ByteString("two".into()),
    ///             BencodedValue::Integer(2),
    ///         ),
    ///         (
    ///             BencodedValue::ByteString("three".into()),
    ///             BencodedValue::Integer(3),
    ///         ),
    ///     ];
    /// assert_eq!(bencoded_byte_string, BencodedValue::Dictionary(dict));
    /// ```
    ///
    fn dictionary(&mut self) -> Result<BencodedValue, ParserError> {
        let mut dict = Vec::new();
        let mut end_found = false;
        self.pop();

        while let Some(c) = self.peek() {
            match c {
                b'1'..=b'9' => {
                    let key = self.byte_string()?;
                    let value = self.bencoded_value()?;
                    dict.push((key, value));
                }
                b'e' => {
                    end_found = true;
                    self.pop();
                    break;
                }
                _ => {
                    return Err(ParserError::InvalidEncoding(
                        self.cursor,
                        "invalid dict element",
                    ))
                }
            }
        }

        if end_found {
            Ok(BencodedValue::Dictionary(dict))
        } else {
            Err(ParserError::InvalidEncoding(self.cursor, "end not found"))
        }
    }
    /// Parses the string if it's a valid bencoded integer and returns
    /// a Result<BencodedValue, ParserError>. Advances the cursor.
    ///
    /// # Errors
    ///
    /// This function will return an error if there an error parsing
    /// the integer value. Both i-0e and i05e (zero must be the only
    /// digit if it is the first digit) will return an error. This
    /// method won't return and error if the *e* delimiter is missing.
    ///
    /// # Examples
    ///
    /// ```#s
    /// let s = "i8e"
    /// let bencoded_dictionary = parse(s)?;
    ///    
    /// assert_eq!(bencoded_byte_string, BencodedValue::Integer(8));
    /// ```
    ///    
    fn integer(&mut self) -> Result<BencodedValue, ParserError> {
        let bencoded_string = self
            .skip(1)
            .take_while(|c| *c != 101)
            .map(|c| c as char)
            .collect::<String>();
        if check_zero(&bencoded_string) {
            return Err(ParserError::InvalidInteger(bencoded_string));
        }

        Ok(BencodedValue::Integer(bencoded_string.parse().map_err(
            |_| ParserError::InvalidInteger(bencoded_string),
        )?))
    }
    /// Parses the string if it's a valid bencoded list and returns a
    /// Result<BencodedValue, ParserError>. Advances the cursor.
    ///
    /// # Errors
    ///
    /// This function will return an error if there an error parsing
    /// any of the elements of the list, or if the *e* delimiter is
    /// missing.
    ///
    /// # Examples
    ///
    /// ```#s
    /// let s = "li1ei2ei3ee";
    /// let list = BencodedValue::List(vec![
    ///                BencodedValue::Integer(1),
    ///                BencodedValue::Integer(2),
    ///                BencodedValue::Integer(3),
    ///      ]);
    /// let bencoded_list = parse(s)?
    /// assert_eq!(bencoded_list, list);
    /// ```
    ///    
    fn list(&mut self) -> Result<BencodedValue, ParserError> {
        let mut vec = Vec::new();
        self.pop();
        let end_found;
        loop {
            if self.take_if(b'e') == Some(b'e') {
                end_found = true;
                break;
            }
            vec.push(self.bencoded_value()?);
        }
        if end_found {
            Ok(BencodedValue::List(vec))
        } else {
            Err(ParserError::InvalidEncoding(self.cursor, "end missing"))
        }
    }
    /// Creates a new instance of parser, that parses the string s
    /// (bencoded string).
    pub fn new(s: Vec<u8>) -> Self {
        Self {
            cursor: 0,
            characters: s,
        }
    }
    /// Parses the string if it's a valid bencoded value and
    /// returns a Result<BencodedValue, ParserError>.
    ///
    /// # Errors
    ///
    /// This function will return an error if there an error parsing
    /// parsing the value, or if the string passed was empty.
    ///
    /// # Examples
    ///
    /// ```#s
    /// let s = "li1ei2ei3ee";
    /// let list = BencodedValue::List(vec![
    ///                BencodedValue::Integer(1),
    ///                BencodedValue::Integer(2),
    ///                BencodedValue::Integer(3),
    ///      ]);
    /// let bencoded_list = parse(s)?
    /// assert_eq!(bencoded_list, list);
    /// ```
    ///   
    pub fn bencoded_value(&mut self) -> Result<BencodedValue, ParserError> {
        self.peek()
            .map(|c| match c {
                b'i' => self.integer(),
                b'1'..=b'9' => self.byte_string(),
                b'l' => self.list(),
                b'd' => self.dictionary(),
                _ => {
                    Err(ParserError::InvalidEncoding(self.cursor, "invalid bencode"))},
            })
            .map_or_else(|| Err(ParserError::Empty), |r| r)
    }

    /// Peeks ahead one character without moving the cursor. Returns
    /// None if there is no more characters.
    fn peek(&self) -> Option<u8> {
        self.characters.get(self.cursor).copied()
    }

    /// Pops the next character, and advances the cursor. Returns None
    /// if there is no more characters.
    fn pop(&mut self) -> Option<u8> {
        self.characters.get(self.cursor).map(|c| {
            self.cursor += 1;
            *c
        })
    }

    /// Pops the next character, advancing the cursor, if it matches,
    /// returns None otherwise.
    fn take_if(&mut self, want: u8) -> Option<u8> {
        if let Some(c) = self.characters.get(self.cursor) {
            if *c == want {
                self.cursor += 1;
                return Some(*c);
            }
        }
        None
    }
}
/// Validates the preconditions of an bencoded integer associated with
/// zero.
fn check_zero(s: &str) -> bool {
    s.contains("-0") || (s.starts_with('0') && s.chars().nth(1).map_or(false, |c| c.is_digit(10)))
}
/// Parses the string if it's a valid bencoded value and
/// returns a Result<BencodedValue, ParserError>.
///
/// # Errors
///
/// This function will return an error if there an error parsing
/// parsing the value, or if the string passed was empty.
///
/// # Examples
///
/// ```#s
///  let s = "lli1ei2ei3eei5e4:testd3:onei1eee";
///  let expected = vec![
///      BencodedValue::List(vec![
///          BencodedValue::Integer(1),
///          BencodedValue::Integer(2),
///          BencodedValue::Integer(3),
///      ]),
///      BencodedValue::Integer(5),
///      BencodedValue::ByteString("test".as_bytes().into()),
///      BencodedValue::Dictionary(vec![(
///          BencodedValue::ByteString("one".into()),
///          BencodedValue::Integer(1),
///      )]),
///  ];
/// let bencoded_value = parse(s)?
/// assert_eq!(value, expected);
/// ```
///   
pub fn parse(s: Vec<u8>) -> Result<BencodedValue, ParserError> {
    let mut parser = Parser::new(s);
    parser.bencoded_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_bencoded_string_for_integer() {
        let s = "i5050e".into();
        assert_eq!(parse(s).unwrap(), BencodedValue::Integer(5050));
    }

    #[test]
    fn parse_bencoded_string_for_negative_integer() {
        let s = "i-55e".into();
        assert_eq!(parse(s).unwrap(), BencodedValue::Integer(-55));
    }

    #[test]
    fn parse_bencoded_string_with_invalid_integer() {
        let s = "i-55ae".into();
        assert_eq!(
            parse(s).unwrap_err(),
            ParserError::InvalidInteger("-55a".into())
        );
    }

    #[test]
    fn parse_bencoded_string_with_negative_zero_returns_error() {
        let s = "i-0e".into();
        assert_eq!(
            parse(s).unwrap_err(),
            ParserError::InvalidInteger("-0".into())
        );
    }
    #[test]
    fn parse_bencoded_string_with_zero_and_other_digit_after_returns_error() {
        let s = "i05e".into();
        assert_eq!(
            parse(s).unwrap_err(),
            ParserError::InvalidInteger("05".into())
        );
    }

    #[test]
    fn parse_bencoded_byte_string() {
        let s = "4:test".into();
        assert_eq!(
            parse(s).unwrap(),
            BencodedValue::ByteString("test".as_bytes().into())
        );
    }

    #[test]
    fn parse_bencoded_byte_string_with_length_more_than_one_digit() {
        let s = "15:more characters".into();
        assert_eq!(
            parse(s).unwrap(),
            BencodedValue::ByteString("more characters".as_bytes().into())
        );
    }

    #[test]
    fn parsing_bencoded_byte_string_with_invalid_length() {
        let s = "4a:aaaa".into();
        assert_eq!(parse(s).unwrap_err(), ParserError::InvalidByteStringLength);
    }

    #[test]
    fn parse_bencoded_list_of_integers() {
        let s = "li1ei2ei3ee".into();
        let integers = (1..=3).map(BencodedValue::Integer).collect();
        assert_eq!(parse(s).unwrap(), BencodedValue::List(integers));
    }

    #[test]
    fn parse_bencoded_list_of_lists() {
        let s = "lli1ei2ei3eeli4ei5ei6eeli7ei8ei9eee".into();
        let lists = vec![
            BencodedValue::List(vec![
                BencodedValue::Integer(1),
                BencodedValue::Integer(2),
                BencodedValue::Integer(3),
            ]),
            BencodedValue::List(vec![
                BencodedValue::Integer(4),
                BencodedValue::Integer(5),
                BencodedValue::Integer(6),
            ]),
            BencodedValue::List(vec![
                BencodedValue::Integer(7),
                BencodedValue::Integer(8),
                BencodedValue::Integer(9),
            ]),
        ];
        assert_eq!(parse(s).unwrap(), BencodedValue::List(lists));
    }

    #[test] // Modify when Dictionary is implemented
    fn parse_bencoded_list_of_different_types() {
        let s = "lli1ei2ei3eei5e4:testd3:onei1eee".into();
        let list = vec![
            BencodedValue::List(vec![
                BencodedValue::Integer(1),
                BencodedValue::Integer(2),
                BencodedValue::Integer(3),
            ]),
            BencodedValue::Integer(5),
            BencodedValue::ByteString("test".as_bytes().into()),
            BencodedValue::Dictionary(vec![(
                BencodedValue::ByteString("one".into()),
                BencodedValue::Integer(1),
            )]),
        ];
        assert_eq!(parse(s).unwrap(), BencodedValue::List(list));
    }

    #[test]
    fn parse_bencoded_empty_list() {
        let s = "le".into();
        let list = vec![];
        assert_eq!(parse(s).unwrap(), BencodedValue::List(list));
    }

    #[test]
    fn parse_bencoded_dictionary_with_integer_values() {
        let s = "d3:onei1e3:twoi2e5:threei3ee".into();
        let mut parser = Parser::new(s);
        let dict = [
            (
                BencodedValue::ByteString("one".into()),
                BencodedValue::Integer(1),
            ),
            (
                BencodedValue::ByteString("two".into()),
                BencodedValue::Integer(2),
            ),
            (
                BencodedValue::ByteString("three".into()),
                BencodedValue::Integer(3),
            ),
        ];

        assert_eq!(
            parser.bencoded_value().unwrap(),
            BencodedValue::Dictionary(dict.into())
        );
    }

    #[test]
    fn parse_bencoded_dictionary_with_different_types() {
        let s = "d3:onei1e6:string3:str4:listli1ei2ei3ee4:dictd3:onei1e3:twoi2eee".into();
        let dict = [
            (
                BencodedValue::ByteString("one".into()),
                BencodedValue::Integer(1),
            ),
            (
                BencodedValue::ByteString("string".into()),
                BencodedValue::ByteString("str".into()),
            ),
            (
                BencodedValue::ByteString("list".into()),
                BencodedValue::List(vec![
                    BencodedValue::Integer(1),
                    BencodedValue::Integer(2),
                    BencodedValue::Integer(3),
                ]),
            ),
            (
                BencodedValue::ByteString("dict".into()),
                BencodedValue::Dictionary(vec![
                    (
                        BencodedValue::ByteString("one".into()),
                        BencodedValue::Integer(1),
                    ),
                    (
                        BencodedValue::ByteString("two".into()),
                        BencodedValue::Integer(2),
                    ),
                ]),
            ),
        ];

        assert_eq!(parse(s).unwrap(), BencodedValue::Dictionary(dict.into()));
    }

    #[test]
    fn parse_bencoded_empty_dictionary() {
        let s = "de".into();
        let dict = BencodedValue::Dictionary(vec![]);

        assert_eq!(parse(s).unwrap(), dict);
    }

    #[test]
    fn parse_bencoded_dictionary_missing_delimiter() {
        let s: Vec<u8> = "d3:onei1e3:twoi2e5:threei3e".into();
        let len = s.len();
        assert_eq!(
            parse(s).unwrap_err(),
            ParserError::InvalidEncoding(len, "end not found")
        )
    }

    #[test]
    fn parse_empty_string() {
        let s = "".into();
        assert_eq!(parse(s).unwrap_err(), ParserError::Empty)
    }

    #[test]
    fn parse_non_bencoded_string() {
        let s = "abc".into();
        assert_eq!(
            parse(s).unwrap_err(),
            ParserError::InvalidEncoding(0, "invalid bencode")
        )
    }

    #[test]
    fn parse_non_utf8_bencoded_bytestring() {
        let s = b"4:\xF0\x9f\x92\x96";
        assert_eq!(
            parse(s.to_vec()).unwrap(),
            BencodedValue::ByteString(vec![240, 159, 146, 150])
        )
    }

    #[test]
    fn parse_dictionary_of_lists() {
        let s = "d1:ali0ee1:bli1ei2eee".into();
        let want = BencodedValue::Dictionary(vec![
            (
                BencodedValue::ByteString("a".into()),
                BencodedValue::List(vec![BencodedValue::Integer(0)]),
            ),
            (
                BencodedValue::ByteString("b".into()),
                BencodedValue::List(vec![BencodedValue::Integer(1), BencodedValue::Integer(2)]),
            ),
        ]);
        assert_eq!(parse(s).unwrap(), want)
    }
}
