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
