use crate::tracker::url_encoder::encoder_error::EncoderError;
use std::fmt::Write;
//use std::str;

#[derive(Debug, PartialEq)]
pub struct URLEncoded(String);

impl Default for URLEncoded {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Creates a new instance of URL encoder.
    pub fn new() -> Self {
        URLEncoded(String::new())
    }

    /// Removes a given char from the string that receives
    /// and returns a ```Vec<String>``` with the secuences of
    /// letters from the string that were separeted by the char.
    ///
    /// # Example
    /// remover("Hi how are you", " ") -> ["Hi","how","are","you"]

    // fn remover(&self, full_str: &str, rem_val: char) -> Vec<String> {
    //     full_str.split(rem_val).map(|s| s.to_string()).collect()
    // }

    /// Returns a string conformed by the elements of the
    /// vector of strings that receives separated by a percentage
    /// and the string that also receives.
    ///
    /// # Example
    /// replacer(["Hi", "how", "are","you"],"%20") -> Hi%20how%20are%20you

    // fn replacer(&self, vec: Vec<String>, encoded_val: &str) -> String {
    //     let mut it = vec.into_iter();
    //     it.next()
    //         .into_iter()
    //         .chain(it.map(|s| format!("%{}{}", encoded_val, s)))
    //         .collect()
    // }

    /// Encodes the recived bytes array using the urlencode algorithm
    /// by calling to remover(), replacer() and encode_hex() and returns it.
    ///
    /// # Example
    /// urlencode("Hi, how are you?") -> Hi%2C%20how%20are%20you%3F

    pub fn urlencode(&self, unencoded_string: &[u8]) -> Result<URLEncoded, EncoderError> {
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

        // let s = match str::from_utf8(&unencoded_string) {
        //     Ok(v) => v.to_string(),
        //     Err(_) => return Err(EncoderError::InvalidUTF8),
        // };
        // let vec_esp_chars = s
        //     .chars()
        //     .filter(|c| !(c.is_digit(36) || *c == '.' || *c == '-' || *c == '_' || *c == '~'))
        //     .collect::<Vec<char>>();

        // if vec_esp_chars.is_empty() {
        //     return Ok(URLEncoded(s));
        // }

        // let mut dst = [0; 1];
        // vec_esp_chars[0].encode_utf8(&mut dst);

        // let mut result_replacer = self.replacer(
        //     self.remover(&s, vec_esp_chars[0]),
        //     &self.encode_hex(&dst)?.to_uppercase(),
        // );

        // for c in vec_esp_chars {
        //     let mut dst = [0; 1];
        //     c.encode_utf8(&mut dst);
        //     result_replacer = self.replacer(
        //         self.remover(&result_replacer, c),
        //         &self.encode_hex(&dst)?.to_uppercase(),
        //     )
        // }

        // Ok(URLEncoded(result_replacer))
    }

    // Returns the hexadecimal value of the vector of bytes that receives.

    // fn encode_hex(&self, bytes: &[u8]) -> Result<String, EncoderError> {
    //     let mut s = String::with_capacity(bytes.len() * 2);
    //     for &b in bytes {
    //         write!(&mut s, "{:02x}", b).map_err(|_| EncoderError::InvalidHexadecimal)?
    //     }
    //     Ok(s)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_url_encoded() {
        let url = URLEncoded::new();

        assert_eq!(
            url.urlencode("Hola".to_string().as_bytes())
                .unwrap()
                .get_url(),
            "Hola".to_string()
        );
    }

    pub fn encoder() -> URLEncoded {
        URLEncoded::new()
    }
    #[test]
    fn encode_phrase_as_urlencode() {
        let string = "Hola, como estas?".to_string();
        assert_eq!(
            encoder().urlencode(&string.as_bytes()).unwrap(),
            URLEncoded("Hola%2C%20como%20estas%3F".to_string())
        );
    }

    #[test]
    fn encode_only_lowercase_letters_without_space_as_urlencode() {
        let string = "abcdefghi".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("abcdefghi".to_string())
        );
    }

    #[test]
    fn encode_only_uppercase_letters_without_space_as_urlencode() {
        let string = "ABCDEFGHI".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("ABCDEFGHI".to_string())
        );
    }

    #[test]
    fn encode_only_numbers_without_space_as_urlencode() {
        let string = "2565467357".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("2565467357".to_string())
        );
    }

    #[test]
    fn encode_only_special_characters_without_space_as_urlencode() {
        let string = "=?$(*+&".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("%3D%3F%24%28%2A%2B%26".to_string())
        );
    }

    #[test]
    fn encode_empty_string_as_urlencode() {
        let string = "".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("".to_string())
        );
    }

    #[test]
    fn encode_dot_as_urlencode() {
        let string = ".".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded(".".to_string())
        );
    }

    #[test]
    fn remove_first_character_from_phrase() {
        let string = " Hola?".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("%20Hola%3F".to_string())
        );
    }

    #[test]
    fn remove_all_characters_from_phrase() {
        let string = "      ".to_string();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded("%20%20%20%20%20%20".to_string())
        );
    }

    #[test]
    fn replace_empty_string_from_phrase() {
        let string = String::new();

        assert_eq!(
            encoder().urlencode(string.as_bytes()).unwrap(),
            URLEncoded(String::new())
        );
    }
}
