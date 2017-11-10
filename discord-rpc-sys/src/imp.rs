#![allow(non_snake_case)]
use libc::{c_char, int64_t, int8_t, c_int};

pub type DiscordReply = c_int;

pub const DISCORD_REPLY_NO: DiscordReply = 0;
pub const DISCORD_REPLY_YES: DiscordReply = 1;
pub const DISCORD_REPLY_IGNORE: DiscordReply = 2;

#[repr(C)]
pub struct DiscordRichPresence {
    pub state: *const c_char,   /* max 128 bytes */ 
    pub details: *const c_char, /* max 128 bytes */
    pub startTimestamp: int64_t,
    pub endTimestamp: int64_t,
    pub largeImageKey: *const c_char,  /* max 32 bytes */
    pub largeImageText: *const c_char, /* max 128 bytes */
    pub smallImageKey: *const c_char,  /* max 32 bytes */
    pub smallImageText: *const c_char, /* max 128 bytes */
    pub partyId: *const c_char,        /* max 128 bytes */
    pub partySize: c_int,
    pub partyMax: c_int,
    pub matchSecret: *const c_char,    /* max 128 bytes */
    pub joinSecret: *const c_char,     /* max 128 bytes */
    pub spectateSecret: *const c_char, /* max 128 bytes */
    pub instance: int8_t,
}

#[repr(C)]
pub struct DiscordJoinRequest {
    pub userId: *const c_char,
    pub username: *const c_char,
    pub avatar: *const c_char,
}

#[repr(C)]
pub struct DiscordEventHandlers {
    pub ready: Option<extern fn()>,
    pub disconnected: Option<extern fn(errorCode: c_int, message: *const c_char)>,
    pub errored: Option<extern fn(errorCode: c_int, message: *const c_char)>,
    pub joinGame: Option<extern fn(joinSecret: *const c_char)>,
    pub spectateGame: Option<extern fn(spectateSecret: *const c_char)>,
    pub joinRequest: Option<extern fn(request: *const DiscordJoinRequest)>,
}

#[link(name = "discord-rpc")]
extern "C" {
    pub fn Discord_Initialize(applicationId: *const c_char,
                        handlers: *mut DiscordEventHandlers,
                        autoRegister: c_int,
                        optionalSteamId: *const c_char);

    pub fn Discord_Shutdown();
    
    pub fn Discord_RunCallbacks();

    pub fn Discord_UpdatePresence(presence: *const DiscordRichPresence);

    pub fn Discord_Respond(userid: *const c_char, reply: DiscordReply);
}

