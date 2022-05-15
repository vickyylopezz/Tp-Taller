/// Represent all the possible types that can be represented in
/// bencode.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BencodedValue {
    ByteString(Vec<u8>),
    Dictionary(Vec<(BencodedValue, BencodedValue)>),
    Integer(i64),
    List(Vec<BencodedValue>),
}
