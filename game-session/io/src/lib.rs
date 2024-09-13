#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};


pub struct GameSessionMetadata;

impl Metadata for GameSessionMetadata {
    type Init = InOut<ActorId, ()>;
    type Handle = InOut<Action, Event>;
    type State = InOut<StateQuery, GameState>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}

#[derive(TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Action {
    StartGame {
        user: ActorId,
    },
    CheckWord {
        user: ActorId,
        word: String,
    },
    CheckGameStatus,
}

#[derive(PartialEq, TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    GameStarted {
        user: ActorId,
    },
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    GameOver { user: ActorId, result: GameResult },
    MessageAlreadySent,
}

#[derive(PartialEq, TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum GameResult {
    Win,
    Lose,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum StateQuery {
    GetGameState,
}

#[derive(PartialEq, TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum SessionStatus {
    Waiting,
    MessageSent,
    ReplyReceived(Event),
}

#[derive(PartialEq, TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum GameStatus {
    NotStarted,
    InProgress,
    GameOver(GameResult),
}

#[derive(PartialEq, TypeInfo, Encode, Decode, Debug, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct GameState {
    pub user: ActorId,
    pub game_status: GameStatus,
    pub session_status: SessionStatus,
}
