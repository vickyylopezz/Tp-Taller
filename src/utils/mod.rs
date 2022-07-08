#[macro_export]
macro_rules! append {
    () => (
	Vec::new()
    );

    ($( $x:expr ), *) => {
	{
	    let mut size = 0;
	    $(
		size += $x.len();
	    )*

	    let mut temp = Vec::with_capacity(size);
	    $(
	    temp.append(&mut $x);
	)*
	    temp
    }
    };
}

pub(crate) use append;
use sha1::{Digest, Sha1};

use crate::torrent::info::{Info, SingleFileData};

pub fn from_u32_be(array: &mut &[u8]) -> Option<u32> {
    let (int_bytes, rest) = array.split_at(std::mem::size_of::<u32>());
    *array = rest;
    Some(u32::from_be_bytes(int_bytes.try_into().ok()?))
}

/// Function made in order to hash an u8 vec.
/// It returns a 20-byte SHA1 hash.
pub fn hash_info(buf: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(buf);
    hasher.finalize()[0..20].try_into().unwrap()
}

pub fn get_info_from_torrentfile(i: Info) -> SingleFileData {
    let Info(mode) = i;
    match mode {
        crate::torrent::info::InfoMode::Empty => todo!(),
        crate::torrent::info::InfoMode::SingleFile(it) => it,
    }
}

pub fn round_float(n: f64, p: usize) -> String {
    format!("{:.1$}", n, p)
}

#[cfg(test)]
mod tests {
    #[test]
    fn append_multiple_vectors() {
        let got = append!(vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]);
        let want = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(got, want)
    }

    #[test]
    fn empty() {
        let got: Vec<u32> = append!();
        let want = vec![];
        assert_eq!(got, want);
    }
}
