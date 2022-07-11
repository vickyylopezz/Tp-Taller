use crate::client::bitfield_error::BitFieldError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    NotDownload,
    InProgress,
    Downloaded,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BitFieldDownload {
    pieces: Vec<Status>,
    /// Number of pieces
    number_pieces: usize,
}

impl BitFieldDownload {
    /// Creates a new [`BitField`] of adequate size for the specified
    /// number of pieces. If the number of pieces isn't a
    /// multiple of eight, the size of the [`BitField`] will be the closest
    /// multiple (greater than the number of pieces).
    pub fn new(number_pieces: usize) -> Result<Self, BitFieldError> {
        if number_pieces == 0 {
            return Err(BitFieldError::InvalidLengthError);
        }
        let pieces = vec![Status::NotDownload; number_pieces];
        Ok(Self {
            pieces,
            number_pieces,
        })
    }

    /// Returns `Some(true)` or `Some(false)` depending if the piece
    /// specified with `piece_index ` was already downloaded or
    /// not. Returns `None` if the index is invalid, this could be
    /// because the index is out of bounds or it doesn't correspond to
    /// a piece, but to padding. The indexes start at zero.
    pub fn has_piece(&self, piece_index: usize) -> Option<bool> {
        if self.number_pieces <= piece_index {
            return None;
        }

        Some(self.pieces[piece_index] == Status::Downloaded)
    }

    pub fn dont_have_piece(&self, piece_index: usize) -> Option<bool> {
        if self.number_pieces <= piece_index {
            return None;
        }

        Some(self.pieces[piece_index] == Status::NotDownload)
    }

    /// Returns `true` or `false` depending if all pieces
    /// were already downloaded or not.
    pub fn has_all_pieces(&self) -> bool {
        for i in 0..self.number_pieces {
            if self.has_piece(i) == Some(false) {
                return false;
            };
        }
        true
    }

    /// Marks the piece specified by `piece_index` as
    /// downloaded. Returns `None` if the index is invalid, this could be
    /// because the index is out of bounds or it doesn't correspond to
    /// a piece, but to padding. The indexes start at zero.
    pub fn set_piece(&mut self, piece_index: usize, status: Status) -> Option<()> {
        if self.number_pieces <= piece_index {
            return None;
        }
        self.pieces[piece_index] = status;
        Some(())
    }

    /// Returns a vector with de indexes of the missing pieces
    pub fn get_missing(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for i in 0..self.number_pieces {
            //eprintln!("{:?}", i);
            if self.dont_have_piece(i) == Some(true) {
                vec.push(i);
            };
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_the_twentieth_piece_as_downloaded() {
        let mut bitfield = BitFieldDownload::new(27).unwrap();
        let previous = bitfield.has_piece(20);
        bitfield.set_piece(20, Status::Downloaded);
        let got = bitfield.has_piece(20);
        let want = Some(true);

        assert_eq!(previous, Some(false));
        assert_eq!(got, want);
    }

    #[test]
    fn trying_to_mark_pieces_that_are_not_present_returns_none() {
        let mut bitfield = BitFieldDownload::new(27).unwrap();
        let previous = bitfield.has_piece(28);
        bitfield.set_piece(28, Status::NotDownload);
        let got = bitfield.has_piece(28);
        let want = None;

        assert_eq!(previous, None);
        assert_eq!(got, want);
    }

    #[test]
    fn create_a_bitfield_with_zero_pieces() {
        let got = BitFieldDownload::new(0);
        println!("got: {:?}", got);
        let want: Option<BitFieldDownload> = None;

        assert_eq!(None, want);
    }
}
