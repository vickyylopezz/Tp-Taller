use crate::client::bitfield_error::BitFieldError;

#[derive(Clone, Eq, PartialEq)]
pub enum Status {
    NotDownload,
    InProgress,
    Downloaded,
}
#[derive(Clone, Eq, PartialEq)]
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn peer_has_the_ninth_piece_of_sixteen() {
//         let bitfield = BitField::new_from_vec(vec![0, 128], 16);
//         let got = bitfield.has_piece(8);
//         let want = Some(true);
//         assert_eq!(got, want);
//     }

//     #[test]
//     fn ask_for_the_fifteenth_piece_when_there_is_only_fourteen() {
//         let bitfield = BitField::new_from_vec(vec![255, 252], 14);
//         let got = bitfield.has_piece(15);
//         let want = None;
//         assert_eq!(got, want);
//     }

//     #[test]
//     fn mark_the_twentieth_piece_as_downloaded() {
//         let mut bitfield = BitField::new(27).unwrap();
//         let previous = bitfield.has_piece(20);
//         bitfield.set_piece(20);
//         let got = bitfield.has_piece(20);
//         let want = Some(true);

//         assert_eq!(previous, Some(false));
//         assert_eq!(got, want);
//     }

//     #[test]
//     fn trying_to_mark_pieces_that_are_not_present_returns_none() {
// 	    let mut bitfield = BitField::new(27).unwrap();
//         let previous = bitfield.has_piece(28);
//         bitfield.set_piece(28);
//         let got = bitfield.has_piece(28);
//         let want = None;

//         assert_eq!(previous, None);
//         assert_eq!(got, want);
//     }

//     #[test]
//     fn create_a_bitfield_with_zero_pieces() {
//         let got = BitField::new(0);
//         println!("got: {:?}", got);
//         let want : Option<BitField> = None;

//         assert_eq!(None, want);
//     }

//     fn get_pieces_left_to_download() {
//         let bitfield = BitField::new_from_vec(vec![0b11111111, 0b10000000, 0b00001111], 23);
//         let got = bitfield.get_missing();
//         let want = vec![9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
//         assert_eq!(got, want);
//     }
// }
