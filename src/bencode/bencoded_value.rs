/// Represent all the possible types that can be represented in
/// bencode.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum BencodedValue {
    /// Bencoded byte string
    ByteString(Vec<u8>),
    /// Bencoded dictionary, the keys and the values are also bencoded
    Dictionary(Vec<(BencodedValue, BencodedValue)>),
    /// Bencoded integers
    Integer(i64),
    /// Bencoded list
    List(Vec<BencodedValue>),
}

impl BencodedValue {
    /// Returns [`Some`] if self coincides with the
    /// [`BencodedValue::ByteString`] variant, else returns [`None`]
    pub fn byte_string(self) -> Option<Vec<u8>> {
        if let BencodedValue::ByteString(s) = self {
            Some(s)
        } else {
            None
        }
    }
    /// Returns [`Some`] if self coincides with the
    /// [`BencodedValue::Dictionary`] variant, else returns [`None`]
    pub fn dictionary(self) -> Option<Vec<(BencodedValue, BencodedValue)>> {
        if let BencodedValue::Dictionary(d) = self {
            Some(d)
        } else {
            None
        }
    }
    /// Returns [`Some`] if self coincides with the
    /// [`BencodedValue::Integer`] variant, else returns [`None`]
    pub fn integer(self) -> Option<i64> {
        if let BencodedValue::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }
    /// Returns [`Some`] if self coincides with the
    /// [`BencodedValue::List`] variant, else returns [`None`]
    pub fn list(self) -> Option<Vec<BencodedValue>> {
        if let BencodedValue::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn encode(&mut self) -> Vec<u8> {
        match self {
            BencodedValue::ByteString(s) => {
                let mut begin = s.len().to_string().into_bytes();
                begin.append(&mut vec![b':']);
                // begin.insert(1, b':');
                begin.append(s);
                begin
            }
            BencodedValue::Dictionary(d) => {
                let mut dict: Vec<u8> = d
                    .iter_mut()
                    .flat_map(|(k, v)| k.encode().into_iter().chain(v.encode().into_iter()))
                    .collect();
                dict.insert(0, b'd');
                dict.push(b'e');
                dict
            }
            BencodedValue::Integer(i) => {
                let mut ascii_integer = i.to_string().into_bytes();
                ascii_integer.insert(0, b'i');
                ascii_integer.push(b'e');
                ascii_integer
            }
            BencodedValue::List(l) => {
                let mut list: Vec<u8> =
		    l.iter_mut().flat_map(|v| v.encode().into_iter()).collect();
                list.insert(0, b'l');
                list.push(b'e');
                list
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enconde_integer() {
        let want = b"i5e";
        let got = BencodedValue::Integer(5).encode();

        assert_eq!(got, want)
    }

    #[test]
    fn enconde_bytestring() {
        let want = b"5:abcde";
        let got = BencodedValue::ByteString(b"abcde".to_vec()).encode();

        assert_eq!(got, want)
    }

    #[test]
    fn enconde_list() {
        let want = b"lli1ei2ei3eei5e4:testd3:onei1eee";
        let got = BencodedValue::List(vec![
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
        ])
        .encode();

        assert_eq!(got, want)
    }
    #[test]
    fn enconde_dictionary() {
        let want = b"d3:onei1e6:string3:str4:listli1ei2ei3ee4:dictd3:onei1e3:twoi2eee";
        let got = BencodedValue::Dictionary(vec![
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
        ])
        .encode();

        assert_eq!(got, want)
    }
}
