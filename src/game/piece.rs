use crate::util::{Element, SCError, SCResult};

use super::{PieceType, Team};

/// A placeable figure on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Type of the (topmost) piece.
    piece_type: PieceType,
    /// Which team this piece belongs to.
    team: Team,
    /// Number of pieces in this castle.
    count: usize,
}

impl Piece {
    /// The type of the (topmost) piece.
    #[inline]
    pub fn piece_type(&self) -> PieceType { self.piece_type }

    /// Which team this piece belongs to.
    #[inline]
    pub fn team(&self) -> Team { self.team }

    /// Number of pieces in this castle.
    #[inline]
    pub fn count(&self) -> usize { self.count }
}

impl TryFrom<&Element> for Piece {
    type Error = SCError;

    fn try_from(elem: &Element) -> SCResult<Self> {
        Ok(Piece {
            piece_type: elem.attribute("type")?.parse()?,
            team: elem.attribute("team")?.parse()?,
            count: elem.attribute("count")?.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{util::Element, game::{Piece, PieceType, Team}};

    #[test]
    fn test_parsing() {
        assert_eq!(Piece::try_from(&Element::from_str(r#"
            <piece type="Herzmuschel" team="TWO" count="1" />
        "#).unwrap()).unwrap(), Piece {
            piece_type: PieceType::Herzmuschel,
            team: Team::Two,
            count: 1,
        });
    }
}
