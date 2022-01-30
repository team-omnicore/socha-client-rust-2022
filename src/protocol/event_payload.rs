use std::fmt;

use crate::{util::{Element, SCResult, SCError}, game::State};

use super::GameResult;

/// The data of a room message from the server.
#[derive(Debug, Clone)]
pub enum EventPayload {
    /// A welcome message by the server.
    Welcome,
    /// A game state.
    Memento(State),
    /// A request by the server to perform a move.
    MoveRequest,
    /// A game result.
    GameResult(GameResult),
}

impl fmt::Display for EventPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Welcome => write!(f, "Welcome"),
            Self::Memento(state) => write!(f, "Memento (turn: {})", state.turn()),
            Self::MoveRequest => write!(f, "MoveRequest"),
            Self::GameResult(result) => write!(f, "GameResult (winner: {})", result.winner().team()),
        }
    }
}

impl TryFrom<&Element> for EventPayload {
    type Error = SCError;

    fn try_from(elem: &Element) -> SCResult<Self> {
        match elem.attribute("class")? {
            "welcomeMessage" => Ok(Self::Welcome),
            "memento" => Ok(Self::Memento(elem.child_by_name("state")?.try_into()?)),
            "moveRequest" => Ok(Self::MoveRequest),
            "result" => Ok(Self::GameResult(elem.try_into()?)),
            _ => Err(SCError::UnknownElement(elem.clone())),
        }
    }
}
