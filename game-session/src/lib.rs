#![no_std]


use gstd::{exec, msg, prelude::*, ActorId, MessageId};
use gstd::debug;
use game_session_io::*;

//const BANK_OF_WORDS: [&str; 3] = ["house", "human", "horse"];
static mut SESSION: Option<GameSession> = None;

//static mut SEED: u8 = 0;

struct GameSession {
    wordle_program: ActorId,
    user: ActorId,
    msg_ids: (SentMessageId, OriginMessageId),
    game_status: GameStatus,
    session_status: SessionStatus,
}


type SentMessageId = MessageId;
type OriginMessageId = MessageId;


#[no_mangle]
extern "C" fn init() {
    debug!("---- RUN Init()");
    let wordle_program: ActorId = msg::load().expect("Unable to decode ActorId");
    unsafe {
        SESSION = Some(GameSession {
            wordle_program,
            user: ActorId::default(),
            msg_ids: (MessageId::zero(), MessageId::zero()),
            game_status: GameStatus::NotStarted,
            session_status: SessionStatus::Waiting,
        });
    }
}


#[no_mangle]
extern "C" fn handle() {
    debug!("---- RUN handle ** ()");
    let action: Action = msg::load().expect("Unable to decode ");
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    // First handle Action::CheckGameStatus to ensure it is executed at any time
    if let Action::CheckGameStatus = action {
        debug!("Checking game status");
        if session.game_status == GameStatus::InProgress || session.session_status == SessionStatus::Waiting {
            session.game_status = GameStatus::NotStarted;
            session.session_status = SessionStatus::Waiting;
            msg::reply(Event::GameOver { user: session.user, result: GameResult::Lose }, 0)
                .expect("Failed to send game over reply");
        }
        return;
    }

    match &session.session_status {
        SessionStatus::Waiting => {
            match action {
                Action::StartGame { user } => {
                    // avoid starting a game if one is already in progress
                    if session.game_status != GameStatus::NotStarted {
                        panic!("Game already in progress");
                    }
                    debug!("#MATCH# Action::StartGame");
                    // StartGame logic here
                    session.user = user;
                    let msg_id = msg::send(session.wordle_program, Action::StartGame { user }, 0)
                        .expect("Unable to send StartGame message");
                    msg::send_delayed(exec::program_id(), Action::CheckGameStatus, 0 , 200 ).expect("Error in sending delayed message.");
                    session.msg_ids = (msg_id, msg::id());
                    session.session_status = SessionStatus::MessageSent;
                    // Wait for the message to be sent
                    exec::wait();
                }
                Action::CheckWord { user, word } => {
                    if session.game_status != GameStatus::InProgress {
                        panic!("No game in progress");
                    }
                    if word.len() != 5 || !word.chars().all(char::is_lowercase) {
                        panic!("Invalid word, must be 5 lowercase letters");
                    }
                    debug!("#MATCH# Action::CheckWord");
                    let msg_id = msg::send(session.wordle_program, Action::CheckWord { user, word }, 0)
                        .expect("Unable to send CheckWord message");
                    session.msg_ids = (msg_id, msg::id());
                    session.session_status = SessionStatus::MessageSent;
                    exec::wait();
                }
                _ => {
                    debug!("---- #MATCH# Action::Other");
                    panic!("Unexpected action");
                }
            }
        }
        SessionStatus::MessageSent => {
            msg::reply(Event::MessageAlreadySent, 0).expect("Error in sending reply");
        }
        SessionStatus::ReplyReceived(reply_message) => {
            debug!("Processing ReplyReceived state with message: {:?}", &reply_message);
            match reply_message.clone() {
                Event::GameStarted { user } => {
                    // msg::reply(Event::Ok, 0).expect("Failed to send reply for Event::Ok");
                    debug!("---- Event::GameStarted sent");
                    session.game_status = GameStatus::InProgress;
                    // It means that the game can be start a new action.
                    session.session_status = SessionStatus::Waiting;
                    debug!("---- Preparing to reply Event::GameStarted for user: {:?}", user);
                    msg::reply(Event::GameStarted { user }, 0).expect("Failed to send reply for Event::GameStarted");
                }
                Event::WordChecked { user, correct_positions, contained_in_word } => {
                    debug!("---- Preparing to reply Event::WordChecked for user: {:?}", user);
                    // It means that the game can be start a new action.
                    session.session_status = SessionStatus::Waiting;
                    let word_guessed = correct_positions.len() == 5;
                    if word_guessed {
                        session.game_status = GameStatus::NotStarted;
                        msg::reply(Event::GameOver { user, result: GameResult::Win }, 0).expect("Failed to send game over reply");
                    } else {
                        msg::reply(Event::WordChecked { user, correct_positions, contained_in_word }, 0).expect("Failed to send word checked reply");
                    }
                }
                _ => {
                    debug!("---- #MATCH# Event::Other");
                    panic!("Unexpected event");
                }
            };
        }
    }
}


#[no_mangle]
extern "C" fn handle_reply() {
    debug!("---- RUN handle_reply()");

    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    if reply_to == session.msg_ids.0 && session.session_status == SessionStatus::MessageSent {
        let reply: Event = msg::load().expect("Unable to decode Event");
        debug!("---- RUN recive reply: {:?}", &reply);
        session.session_status = SessionStatus::ReplyReceived(reply);
        exec::wake(session.msg_ids.1).expect("Failed to wake message");
    }

}


#[no_mangle]
extern "C" fn state() {
    debug!("---- RUN state()");
    let query: StateQuery = msg::load().expect("Unable to decode StateQuery");
    let session = unsafe { SESSION.as_ref().expect("The session is not initialized") };

    match query {
        StateQuery::GetGameState => {
            let game_state = GameState {
                user: session.user,
                game_status: session.game_status.clone(),
                session_status: session.session_status.clone(),
            };
            msg::reply(game_state, 0).expect("Failed to send game state");
        }
    }
}
