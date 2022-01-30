use std::fmt;

use crate::util::{Element, SCError, SCResult};

/// A position on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coords {
    x: usize,
    y: usize,
}

impl Coords {
    #[inline]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> usize { self.x }

    #[inline]
    pub fn y(&self) -> usize { self.y }
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl TryFrom<&Element> for Coords {
    type Error = SCError;

    fn try_from(elem: &Element) -> SCResult<Self> {
        Ok(Coords::new(elem.attribute("x")?.parse()?, elem.attribute("y")?.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{util::Element, game::Coords};

    #[test]
    fn test_parsing() {
        assert_eq!(Coords::try_from(&Element::from_str(r#"
            <coords x="23" y="0" />
        "#).unwrap()).unwrap(), Coords::new(23, 0));
    }
}
