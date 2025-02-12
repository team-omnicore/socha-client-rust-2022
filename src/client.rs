use crate::game::{Move, State, Team};
use crate::protocol::{Event, EventPayload, GameResult, Request, RequestPayload};
use crate::util::{Element, SCError, SCResult};
use log::{debug, error, info, warn};
use quick_xml::events::{BytesEnd, BytesStart, Event as XmlEvent};
use quick_xml::{Reader, Writer};
use std::convert::TryFrom;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

/// A handler that implements the game player's
/// behavior, usually employing some custom move
/// selection strategy.
pub trait SCClientDelegate {
    /// Invoked whenever the game state updates.
    fn on_update_state(&mut self, _state: &State) {}

    /// Invoked when the game ends.
    fn on_game_end(&mut self, _result: &GameResult, _my_team: Team) {}

    /// Invoked when the welcome message is received
    /// with the player's team.
    fn on_welcome(&mut self, _team: Team) {}

    /// Requests a move from the delegate. This method
    /// should implement the "main" game logic.
    fn request_move(&mut self, state: &State, my_team: Team) -> Move;
}

/// A configuration that determines whether
/// the reader and/or the writer of a stream
/// should be swapped by stdio to ease debugging.
pub struct DebugMode {
    pub debug_reader: bool,
    pub debug_writer: bool,
}

/// The client which handles XML requests, manages
/// the game state and invokes the delegate.
pub struct SCClient<D>
where
    D: SCClientDelegate,
{
    delegate: D,
    debug_mode: DebugMode,
    reservation_code: Option<String>,
    client_team: Option<Team>, // TODO: Add game state
}

impl<D> SCClient<D>
where
    D: SCClientDelegate,
{
    /// Creates a new client using the specified delegate.
    pub fn new(delegate: D, debug_mode: DebugMode, reservation_code: Option<String>) -> Self {
        Self {
            delegate,
            debug_mode,
            reservation_code,
            client_team: None,
        }
    }

    /// Blocks the thread and begins reading XML messages
    /// from the provided address via TCP.
    pub fn connect(&mut self, host: &str, port: u16) -> SCResult<GameResult> {
        let address = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&address)?;
        info!("Connected to {}", address);

        // Begin parsing game messages from the stream.
        // List all combinations of modes explicitly,
        // since they generate different generic instantiations
        // of `run_game`.

        let mode = &self.debug_mode;
        let game_result = if mode.debug_reader && !mode.debug_writer {
            self.run(io::stdin(), stream)?
        } else if !mode.debug_reader && mode.debug_writer {
            self.run(stream, io::stdout())?
        } else if mode.debug_reader && mode.debug_writer {
            self.run(io::stdin(), io::stdout())?
        } else {
            self.run(stream.try_clone()?, stream)?
        };

        Ok(game_result)
    }

    /// Blocks the thread and parses/handles game messages
    /// from the provided reader.
    fn run(&mut self, read: impl Read, write: impl Write) -> SCResult<GameResult> {
        let mut buf = Vec::new();
        let mut reader = Reader::from_reader(BufReader::new(read));
        let mut writer = Writer::new(BufWriter::new(write));

        // Write <protocol>
        writer.write_event(XmlEvent::Start(BytesStart::borrowed_name(b"protocol")))?;

        // Send join request
        let join_xml: Element = match &self.reservation_code {
            Some(code) => Request::JoinPrepared {
                reservation_code: code.to_owned(),
            },
            None => Request::Join,
        }
        .into();
        info!("Sending join request {}", &join_xml);
        join_xml.write_to(&mut writer)?;

        // Read <protocol>
        loop {
            match reader.read_event(&mut buf)? {
                XmlEvent::Start(ref start) if start.name() == b"protocol" => {
                    info!("Performed handshake");
                    break;
                }
                XmlEvent::Text(_) => (),
                XmlEvent::Eof => return Err(SCError::Eof),
                e => warn!("Got unexpected event {:?}", e),
            }
        }

        // Handle events from the server
        let mut state: Option<State> = None;
        let mut game_result: Option<GameResult> = None;
        loop {
            let event_xml = Element::read_from(&mut reader)?;

            debug!("Got event {}", event_xml);
            match Event::try_from(&event_xml) {
                Ok(Event::Joined { room_id }) => {
                    info!("Joined room {}", room_id);
                }
                Ok(Event::Left { room_id }) => {
                    info!("Left room {}", room_id);
                    writer.write_event(XmlEvent::Empty(BytesStart::borrowed_name(
                        b"sc.protocol.CloseConnection",
                    )))?;
                    writer.write_event(XmlEvent::End(BytesEnd::borrowed(b"protocol")))?;
                    debug!("Wrote close connection");
                    break;
                }
                Ok(Event::Room { room_id, payload }) => {
                    debug!("Got {} in room {}", payload, room_id);
                    match payload {
                        EventPayload::Welcome(team) => {
                            self.delegate.on_welcome(team);
                            self.client_team = Some(team);
                        }
                        EventPayload::GameResult(result) => {
                            self.delegate
                                .on_game_end(&result, self.client_team.unwrap());
                            game_result = Some(result);
                        }
                        EventPayload::Memento(new_state) => {
                            self.delegate.on_update_state(&new_state);
                            state = Some(new_state);
                        }
                        EventPayload::MoveRequest => {
                            let state = state.as_ref().ok_or_else(|| {
                                SCError::InvalidState(
                                    "No state available at move request!".to_owned(),
                                )
                            })?;
                            let team = state.current_team().ok_or_else(|| {
                                SCError::InvalidState(
                                    "No team available at move request!".to_owned(),
                                )
                            })?;
                            let new_move = self.delegate.request_move(state, team);
                            let request = Request::Room {
                                room_id,
                                payload: RequestPayload::Move(new_move),
                            };
                            let request_xml = Element::from(request);
                            request_xml.write_to(&mut writer)?;
                        }
                    };
                }
                Err(SCError::UnknownElement(element)) => {
                    warn!("Got unknown tag <{}>: {}", element.name(), element);
                }
                Err(SCError::ServerError(message)) => {
                    error!("Server error: {}", message);
                }
                Err(e) => {
                    warn!("Error while parsing event: {:?}", e);
                }
            }
        }

        sleep(Duration::from_secs(2));

        if let Some(result) = game_result {
            Ok(result)
        } else {
            Err(SCError::InvalidState(
                "Failed to receive game_result".to_string(),
            ))
        }
    }

    /// Return team of the client
    pub fn team(&self) -> Option<Team> {
        self.client_team.clone()
    }

    /// Return reservation code, if any
    pub fn reservation(&self) -> Option<String> {
        self.reservation_code.clone()
    }
}
