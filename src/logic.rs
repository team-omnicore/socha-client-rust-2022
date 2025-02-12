use log::info;
use rand::seq::SliceRandom;

use crate::{
    client::SCClientDelegate,
    game::{Move, State, Team},
};

/// An empty game logic structure that
/// implements the client delegate trait
/// and thus is responsible e.g. for picking
/// a move when requested.
pub struct OwnGameLogic;

impl SCClientDelegate for OwnGameLogic {
    fn request_move(&mut self, state: &State, _my_team: Team) -> Move {
        info!("Requested move");
        let chosen_move = *state
            .possible_moves()
            .choose(&mut rand::thread_rng())
            .expect("No move found!");
        info!("Chose move {}", chosen_move);
        chosen_move
    }
}
