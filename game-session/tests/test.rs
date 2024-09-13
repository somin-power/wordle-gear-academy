use gstd::ActorId;
use gtest::{Log, Program, System};
use game_session_io::*;

const USER: u64 = 3;
const TARGET_PROGRAME_ADDRESS: u64 = 2;


#[test]
fn test() {
    let system = System::new();
    system.init_logger();

    let proxy_program = Program::current(&system);

    let target_program = Program::from_file(&system, "../target/wasm32-unknown-unknown/debug/wordle.opt.wasm");
    let result = target_program.send_bytes(USER, []);
    assert!(!result.main_failed());

    println!("target_program = {:?}", target_program.id());

    let target_program_address: ActorId = TARGET_PROGRAME_ADDRESS.into();
    let result = proxy_program.send(USER, target_program_address);
    assert!(!result.main_failed());

    // Send a message to start the game Action::StartGame
    let result = proxy_program.send(USER, Action::StartGame { user: USER.into() });
    assert!(!result.main_failed());

    println!(" Check result = {:?}", Event::GameStarted { user: USER.into() });
    let log = Log::builder().source(1).dest(USER).payload(Event::GameStarted { user: USER.into() });
    // let log = Log::builder().source(1).dest(USER).payload(Event::Ok);
    println!("result = {:?}", &result);
    println!("log = {:?}", &log);
    assert!(result.contains(&log));

    // Send a message to check the word
    let result = proxy_program.send(USER, Action::CheckWord { user: USER.into(), word: "hxxxx".to_string() });
    assert!(!result.main_failed());

    // Check the logs to ensure the word was checked correctly
    let log = Log::builder().source(1).dest(USER).payload(Event::WordChecked { user: USER.into(), correct_positions: vec![0], contained_in_word: vec![] });
    assert!(result.contains(&log));

    // Test state by proxy_program.read_state()
    let wordle_program = system.get_program(1);

    let state: GameState = wordle_program.as_ref().expect("REASON").read_state(StateQuery::GetGameState).expect("Unexpected invalid state.");
    assert_eq!(state.game_status, GameStatus::InProgress);
    assert_eq!(state.session_status, SessionStatus::Waiting);

    // let _result = system.spend_blocks(200);
    // let state: GameState = wordle_program.expect("REASON").read_state(StateQuery::GetGameState).expect("Unexpected invalid state.");
    // assert_eq!(state.game_status, GameStatus::NotStarted);
    // assert_eq!(state.session_status, SessionStatus::Waiting);

}

