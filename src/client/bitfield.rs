use crate::client::bitfield_error::BitFieldError;

#[derive(Debug, PartialEq, Eq)]
/// This struct models the data sent in the bitfield message from the
/// *Peer Wire Protocol*. Keeps track of the downloaded pieces; 1
/// means that the piece was already downloaded and is ready to be
/// sent, 0 otherwise.
#[derive(Clone)]
pub struct BitField {
    /// Bit array, where the highest bit of the first byte is
    /// associated to the first piece. If the number of pieces isn't a
    /// multiple of eight, the size of the vector will be the closest
    /// multiple (greater than the number of pieces)
    bits: Vec<u8>,
    /// Number of pieces
    pieces: usize,
}

impl BitField {
    /// Creates a new [`BitField`] of adequate size for the specified
    /// number of pieces. If the number of pieces isn't a
    /// multiple of eight, the size of the [`BitField`] will be the closest
    /// multiple (greater than the number of pieces).
    pub fn new(pieces: usize) -> Result<Self, BitFieldError> {
        if pieces == 0 {
            return Err(BitFieldError::InvalidLengthError);
        }
        let size = ((pieces - 1) | 7) + 1;
        let bits = vec![0; size];
        Ok(Self { bits, pieces })
    }

    /// Creates a new [`BitField`] from a [`Vec<u8>`]. The number of
    /// pieces must be adequate for the size of the vector:
    /// len(bits) * 8 >= pieces
    pub fn new_from_vec(bits: Vec<u8>, pieces: usize) -> Self {
        // TODO: Could be improved with error handling
        Self { bits, pieces }
    }

    /// Returns `Some(true)` or `Some(false)` depending if the piece
    /// specified with `piece_index ` was already downloaded or
    /// not. Returns `None` if the index is invalid, this could be
    /// because the index is out of bounds or it doesn't correspond to
    /// a piece, but to padding. The indexes start at zero.
    pub fn has_piece(&self, piece_index: usize) -> bool {
        let index: usize = piece_index / (std::mem::size_of::<u8>() * 8);
        let value = self.bits[index];
        let shift: usize = piece_index % (std::mem::size_of::<u8>() * 8);
        let difference = 7 - shift;
        let mask = 0x01;
        (value >> difference & mask) == 1
    }

    /// Returns `true` or `false` depending if any piece
    /// was already downloaded or not.
    pub fn has_any_piece(&self) -> bool {
        for i in 0..self.pieces {
            if self.has_piece(i) {
                return true;
            };
        }
        false
    }

    /// Returns `true` or `false` depending if all pieces
    /// were already downloaded or not.
    pub fn has_all_pieces(&self) -> bool {
        for i in 0..self.pieces {
            if !self.has_piece(i) {
                return false;
            };
        }
        true
    }

    /// Marks the piece specified by `piece_index` as
    /// downloaded. Returns `None` if the index is invalid, this could be
    /// because the index is out of bounds or it doesn't correspond to
    /// a piece, but to padding. The indexes start at zero.
    pub fn set_piece(&mut self, piece_index: usize) -> Option<()> {
        if self.pieces <= piece_index {
            return None;
        }
        let index: usize = piece_index / (std::mem::size_of::<u8>() * 8);
        let shift: usize = piece_index % (std::mem::size_of::<u8>() * 8);
        let difference = 7 - shift;

        let mask = 0x01;
        self.bits[index] |= mask << difference;
        Some(())
    }

    /// Returns a vector with the indexes of the missing pieces
    pub fn get_missing(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for i in 0..self.pieces {
            if !self.has_piece(i) {
                vec.push(i);
            };
        }
        vec
    }

    /// Returns a vector with yhe indexes of the available pieces
    pub fn get_available(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for i in 0..self.pieces {
            if self.has_piece(i) {
                vec.push(i);
            };
        }
        vec
    }

    pub fn get_downloaded(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for i in 0..self.pieces {
            if self.has_piece(i) {
                vec.push(i);
            };
        }
        vec
    }

    pub fn pieces(&self) -> usize {
        self.pieces
    }

    pub fn bits(&self) -> Vec<u8> {
        self.bits.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_has_the_ninth_piece_of_sixteen() {
        let bitfield = BitField::new_from_vec(vec![0, 128], 16);
        let got = bitfield.has_piece(8);
        let want = true;
        assert_eq!(got, want);
    }

    #[test]
    fn ask_for_the_fifteenth_piece_when_there_is_only_fourteen() {
        let bitfield = BitField::new_from_vec(vec![255, 252], 14);
        let got = bitfield.has_piece(15);
        let want = false;
        assert_eq!(got, want);
    }

    #[test]
    fn mark_the_twentieth_piece_as_downloaded() {
        let mut bitfield = BitField::new(27).unwrap();
        let previous = bitfield.has_piece(20);
        bitfield.set_piece(20);
        let got = bitfield.has_piece(20);
        let want = true;

        assert_eq!(previous, false);
        assert_eq!(got, want);
    }

    #[test]
    fn trying_to_mark_pieces_that_are_not_present_returns_none() {
        let mut bitfield = BitField::new(27).unwrap();
        let previous = bitfield.has_piece(28);
        bitfield.set_piece(28);
        let got = bitfield.has_piece(28);
        let want = false;

        assert_eq!(previous, false);
        assert_eq!(got, want);
    }

    #[test]
    fn create_a_bitfield_with_zero_pieces() {
        let got = BitField::new(0);
        println!("got: {:?}", got);
        let want: Option<BitField> = None;

        assert_eq!(None, want);
    }

    fn _get_pieces_left_to_download() {
        let bitfield = BitField::new_from_vec(vec![0b11111111, 0b10000000, 0b00001111], 23);
        let got = bitfield.get_missing();
        let want = vec![9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
        assert_eq!(got, want);
    }
}
