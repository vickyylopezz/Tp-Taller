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
}
