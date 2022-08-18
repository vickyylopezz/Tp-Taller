use crate::tracker::url_encoder::encoder_error::EncoderError;
use std::fmt::Write;
//use std::str;

#[derive(Debug, PartialEq)]
pub struct URLEncoded(pub String);

impl URLEncoded {
    /// Returns the url saved by the struct.
    /// # Examples
    ///
    /// ```#s
    /// let url = URLEncoded::new();
    ///
    /// assert_eq!(url.urlencode("Hola".to_string().as_bytes()).get_url(), "Hola".to_string());
    /// ```
    pub fn get_url(&self) -> String {
        String::from(&self.0)
    }

    /// Encodes the recived bytes array using the urlencode algorithm
    /// by calling to remover(), replacer() and encode_hex() and returns it.
    ///
    /// # Example
    /// urlencode("Hi, how are you?") -> Hi%2C%20how%20are%20you%3F

    pub fn encode(unencoded_string: &[u8]) -> Result<Self, EncoderError> {
        let mut out: String = String::new();

        for byte in unencoded_string.iter() {
            match *byte as char {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '.' | '-' | '_' | '~' => {
                    out.push(*byte as char)
                }
                _ => write!(&mut out, "%{:02X}", byte).unwrap(),
            };
        }

        Ok(URLEncoded(out))
    }

    // has to be valid utf8
    pub fn decode(self) -> Option<Vec<u8>> {
        let s = self.0;
        decode(s)
    }
}

fn decode(s: String) -> Option<Vec<u8>> {
    let chars = s.into_bytes();
    let mut itr = chars.iter();
    let mut bytes = Vec::with_capacity(chars.len()); // at least the length of the string
    while let Some(c) = itr.next() {
        match c {
            b'%' => {
                let val = itr
                    .next()
                    .into_iter()
                    .chain(itr.next().into_iter())
                    .copied()
                    .collect::<Vec<u8>>();
                let b = u8::from_str_radix(&String::from_utf8_lossy(&val), 16).ok()?;
                bytes.push(b);
            }
            _ => bytes.push(*c),
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_url_encoded() {
        assert_eq!(
            URLEncoded::encode("Hola".to_string().as_bytes())
                .unwrap()
                .get_url(),
            "Hola".to_string()
        );
    }

    #[test]
    fn decode_urlencoded() {
        let s1 = URLEncoded::encode(b"Hola, como estas?").unwrap();
        let s2 = URLEncoded::encode(b"=?$(*+&").unwrap();
        let s3 = URLEncoded::encode("α β γ".as_bytes()).unwrap();

        assert_eq!(s1.decode(), Some(b"Hola, como estas?".to_vec()));
        assert_eq!(s2.decode(), Some(b"=?$(*+&".to_vec()));
        assert_eq!(s3.decode(), Some("α β γ".as_bytes().into()));
    }

    #[test]
    fn encode_phrase_as_urlencode() {
        let string = "Hola, como estas?".to_string();
        assert_eq!(
            URLEncoded::encode(&string.as_bytes()).unwrap(),
            URLEncoded("Hola%2C%20como%20estas%3F".to_string())
        );
    }

    #[test]
    fn encode_only_lowercase_letters_without_space_as_urlencode() {
        let string = "abcdefghi".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("abcdefghi".to_string())
        );
    }

    #[test]
    fn encode_only_uppercase_letters_without_space_as_urlencode() {
        let string = "ABCDEFGHI".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("ABCDEFGHI".to_string())
        );
    }

    #[test]
    fn encode_only_numbers_without_space_as_urlencode() {
        let string = "2565467357".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("2565467357".to_string())
        );
    }

    #[test]
    fn encode_only_special_characters_without_space_as_urlencode() {
        let string = "=?$(*+&".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("%3D%3F%24%28%2A%2B%26".to_string())
        );
    }

    #[test]
    fn encode_empty_string_as_urlencode() {
        let string = "".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("".to_string())
        );
    }

    #[test]
    fn encode_dot_as_urlencode() {
        let string = ".".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded(".".to_string())
        );
    }

    #[test]
    fn remove_first_character_from_phrase() {
        let string = " Hola?".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("%20Hola%3F".to_string())
        );
    }

    #[test]
    fn remove_all_characters_from_phrase() {
        let string = "      ".to_string();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded("%20%20%20%20%20%20".to_string())
        );
    }

    #[test]
    fn replace_empty_string_from_phrase() {
        let string = String::new();

        assert_eq!(
            URLEncoded::encode(string.as_bytes()).unwrap(),
            URLEncoded(String::new())
        );
    }
}
