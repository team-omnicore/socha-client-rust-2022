use crate::util::{Element, SCError, SCResult};

/// The state of the game at a point in time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    // TODO
}

impl TryFrom<&Element> for State {
    type Error = SCError;

    fn try_from(elem: &Element) -> SCResult<Self> {
        // TODO
        Ok(State {})
    }
}
