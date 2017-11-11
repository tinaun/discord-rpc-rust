//! event handling

#[derive(Debug, Clone)]
pub struct JoinRequest {
    pub id: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// a reply to a join request
pub enum Reply {
    No,
    Yes,
    Ignore,
}

#[derive(Debug)]
pub enum Event {
    Ready,
    Disconnected(i32, String),
    Errored(i32, String),
    JoinGame(String),
    SpectateGame(String),
    JoinRequest(JoinRequest),
}
