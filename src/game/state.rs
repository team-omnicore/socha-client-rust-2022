use std::{collections::HashMap, str::FromStr};

use crate::util::{Element, SCError, SCResult};

use super::{Board, Move, Team};

/// The state of the game at a point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// The game board.
    board: Board,
    /// The ambers per team.
    ambers: HashMap<Team, usize>,
    /// The turn of the game.
    turn: usize,
    /// The most recent move.
    last_move: Option<Move>,
    /// The starting team.
    start_team: Option<Team>,
}

impl TryFrom<&Element> for State {
    type Error = SCError;

    fn try_from(elem: &Element) -> SCResult<Self> {
        Ok(State {
            board: elem.child_by_name("board")?.try_into()?,
            ambers: elem
                .child_by_name("ambers")?
                .childs_by_name("entry")
                .map(|e| {
                    let team = Team::from_str(e.child_by_name("team")?.content())?;
                    let piece = usize::from_str(e.child_by_name("int")?.content())?;
                    Ok((team, piece))
                })
                .collect::<SCResult<_>>()?,
            turn: elem.attribute("turn")?.parse()?,
            last_move: elem.child_by_name("lastMove").ok().and_then(|m| m.try_into().ok()),
            start_team: elem.child_by_name("startTeam").ok().and_then(|t| t.content().parse().ok()),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{util::Element, game::{Board, State, Team}, hashmap};

    #[test]
    fn test_parsing() {
        assert_eq!(State::try_from(&Element::from_str(r#"
            <state turn="3">
                <board>
                    <pieces></pieces>
                </board>
                <ambers>
                    <entry>
                        <team>ONE</team>
                        <int>1</int>
                    </entry>
                    <entry>
                        <team>TWO</team>
                        <int>0</int>
                    </entry>
                </ambers>
            </state>
        "#).unwrap()).unwrap(), State {
            board: Board::empty(),
            ambers: hashmap![
                Team::One => 1usize,
                Team::Two => 0usize
            ],
            last_move: None,
            start_team: None,
            turn: 3,
        });
    }
}
