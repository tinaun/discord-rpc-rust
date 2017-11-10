#![allow(non_snake_case)]

use std::sync::mpsc;
use std::sync::Mutex;

use events::{Event, JoinRequest, Reply};

use std::ffi::{CString, CStr};
use sys;
use std::ptr;
use libc::{int64_t, c_int, int8_t, c_char};

// this is a hack to get around c callbacks
lazy_static! {
    static ref CALLBACKS: Mutex<Option<RichPresenceCallbacks>> = Default::default();
}

extern fn evh_ready() {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        (rpc.ready_callback)();
    }
}

extern fn evh_disconnected(errorCode: c_int, message: *const c_char) {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        let error_code = errorCode as i32;
        unsafe {
            let message = CStr::from_ptr(message).to_string_lossy().into_owned();

            (rpc.disconnected_callback)(error_code, message);
        }
    }
}

extern fn evh_errored(errorCode: c_int, message: *const c_char) {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        let error_code = errorCode as i32;
        unsafe {
            let message = CStr::from_ptr(message).to_string_lossy().into_owned();
            
            (rpc.errored_callback)(error_code, message);
        }
    }
}

extern fn evh_join(joinSecret: *const c_char) {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        unsafe {
            let message = CStr::from_ptr(joinSecret).to_string_lossy().into_owned();
            
            (rpc.join_callback)(message);
        }
    }
}

extern fn evh_spectate(spectateSecret: *const c_char) {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        unsafe {
            let message = CStr::from_ptr(spectateSecret).to_string_lossy().into_owned();
            
            (rpc.spectate_callback)(message);
        }
    }
}

extern fn evh_joinreq(request: *const sys::DiscordJoinRequest) {
    if let Some(ref mut rpc) = *CALLBACKS.lock().unwrap() {
        unsafe {
            let id = CStr::from_ptr((*request).userId).to_string_lossy().into_owned();
            let name = CStr::from_ptr((*request).username).to_string_lossy().into_owned();
            let avatar = CStr::from_ptr((*request).avatar).to_string_lossy().into_owned();

            let jr = JoinRequest {
                id, name, avatar,
            };
            
            (rpc.join_request)(jr);
        }
    }
}

/// a rpc connection providing a rich presence inside of discord
pub struct RichPresence {
    app_id: CString,
    steam_id: Option<CString>,
    c_cbs: sys::DiscordEventHandlers, 
    config: RichPresenceConfig,
    queue: mpsc::Receiver<Event>,
}

impl RichPresence {
    /// create a connection
    pub fn init(app_id: &str, auto_register: bool, steam_id: Option<&str>) -> Self {
        let app_id = CString::new(app_id).unwrap();
        let steam_id = steam_id.map(|c| CString::new(c).unwrap());

        let (send, recv) = mpsc::channel();

        let s = send.clone();
        let ready_callback = Box::new(move || {
            let _ = s.send(Event::Ready);
        });

        let s = send.clone(); 
        let disconnected_callback = Box::new(move |ec, msg| {
            let _ = s.send(Event::Disconnected(ec, msg));
        });

        let s = send.clone(); 
        let errored_callback = Box::new(move |ec, msg| {
            let _ = s.send(Event::Errored(ec, msg));
        });

        let s = send.clone(); 
        let join_callback = Box::new(move |secret| {
            let _ = s.send(Event::JoinGame(secret));
        });

        let s = send.clone(); 
        let spectate_callback = Box::new(move |secret| {
            let _ = s.send(Event::SpectateGame(secret));
        });

        let s = send.clone(); 
        let join_request = Box::new(move |jr| {
            let _ = s.send(Event::JoinRequest(jr));
        });

        let cbs = RichPresenceCallbacks {
            ready_callback,
            disconnected_callback,
            errored_callback,
            join_callback,
            spectate_callback,
            join_request,
        };

        *CALLBACKS.lock().unwrap() = Some(cbs);

        let c_cbs = sys::DiscordEventHandlers {
            ready: Some(evh_ready),
            disconnected: Some(evh_disconnected),
            errored: Some(evh_errored),
            joinGame: Some(evh_join),
            spectateGame: Some(evh_spectate),
            joinRequest: Some(evh_joinreq),
        };

        let mut rp = RichPresence {
            app_id,
            steam_id,
            c_cbs,
            config: Default::default(),
            queue: recv,
        };

        unsafe {
            sys::Discord_Initialize(
                rp.app_id.as_ptr(),
                &mut rp.c_cbs as *mut _,
                auto_register as c_int,
                rp.steam_id.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null())
            );
        }

        rp
    }

    /// get events 
    pub fn try_recv_event(&self) -> Option<Event> {
        use std::sync::mpsc::TryRecvError;

        match self.queue.try_recv() {
            Ok(e) => Some(e),
            Err(TryRecvError::Empty) => {
                unsafe {
                    sys::Discord_RunCallbacks();
                }
                None
            }
            _ => None
        }
    }

    /// forces all queued presence updates to go through
    pub fn send_update(&self) {
        let config = self.config.as_c_config();
        unsafe {
            sys::Discord_UpdatePresence(&config as *const _);
        }
    }

    /// after a user has asked to join, confirm or deny their request
    /// 
    pub fn send_reply(&self, user_id: &str, reply: Reply) {
        unsafe {
            let c_user_id = CString::new(user_id).ok().map(|c| c.as_ptr()).unwrap_or(ptr::null());

            sys::Discord_Respond( 
                c_user_id,
                reply as sys::DiscordReply,
            )
        }
    }

    /// sets the main status of the game
    /// 
    /// 128 bytes max
    pub fn set_status(&mut self, status: &str) {
        if status.len() > 128 || status.len() == 0 {
            self.config.state = None;
        } else {
            self.config.state = CString::new(status).ok();
        }

        //self.send_update();
    }

    /// sets secondary, detailed status updates
    /// 
    pub fn set_details(&mut self, details: &str) {
        if details.len() > 128 || details.len() == 0 {
            self.config.details = None;
        } else {
            self.config.details = CString::new(details).ok();
        }

        //self.send_update();
    }

    /// the start and end of the current gameplay session
    ///
    pub fn set_duration(&mut self, start: i64, end: i64) {
        self.config.start_timestamp = start;
        self.config.end_timestamp = end;

        //self.send_update();
    }

    /// sets users small profile image and hover tooltip
    /// 
    pub fn set_small_image(&mut self, image_key: &str, tooltip: &str) {
        if image_key.len() > 32 || image_key.len() == 0 {
            self.config.small_image_key = None;
        } else {
            self.config.small_image_text = CString::new(image_key).ok();
        }

        if tooltip.len() > 128 || tooltip.len() == 0 {
            self.config.small_image_text = None;
        } else {
            self.config.small_image_text = CString::new(tooltip).ok();
        }

        //self.send_update();
    }   

    /// sets users small profile image and hover tooltip
    /// 
    pub fn set_large_image(&mut self, image_key: &str, tooltip: &str) {
        if image_key.len() > 32 || image_key.len() == 0 {
            self.config.large_image_key = None;
        } else {
            self.config.large_image_key = CString::new(image_key).ok();
        }

        if tooltip.len() > 128 || tooltip.len() == 0 {
            self.config.large_image_text = None;
        } else {
            self.config.large_image_text = CString::new(tooltip).ok();
        }

        //self.send_update();
    }   
}

impl Drop for RichPresence {
    fn drop(&mut self) {
        *CALLBACKS.lock().unwrap() = None;

        unsafe {
            sys::Discord_Shutdown();
        }
    }
}

struct RichPresenceCallbacks {
    ready_callback: Box<FnMut() + Send>,
    disconnected_callback: Box<FnMut(i32, String) + Send>,
    errored_callback: Box<FnMut(i32, String) + Send>,
    join_callback: Box<FnMut(String) + Send>,
    spectate_callback: Box<FnMut(String) + Send>,
    join_request: Box<FnMut(JoinRequest) + Send>,
}

#[derive(Debug, Default)]
struct RichPresenceConfig {
    pub state: Option<CString>,   /* max 128 bytes */ 
    pub details: Option<CString>, /* max 128 bytes */
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub large_image_key: Option<CString>,  /* max 32 bytes */
    pub large_image_text: Option<CString>, /* max 128 bytes */
    pub small_image_key: Option<CString>,  /* max 32 bytes */
    pub small_image_text: Option<CString>, /* max 128 bytes */
    pub party_id: Option<CString>,        /* max 128 bytes */
    pub party_size: i32,
    pub party_max: i32,
    pub match_secret: Option<CString>,    /* max 128 bytes */
    pub join_secret: Option<CString>,     /* max 128 bytes */
    pub spectate_secret: Option<CString>, /* max 128 bytes */
    pub instance: bool,
}

impl RichPresenceConfig {
    fn as_c_config(&self) -> sys::DiscordRichPresence {
        sys::DiscordRichPresence {
            state: self.state.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            details: self.details.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            startTimestamp: self.start_timestamp as int64_t,
            endTimestamp: self.end_timestamp as int64_t,
            largeImageKey: self.large_image_key.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            largeImageText: self.large_image_text.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            smallImageKey: self.small_image_key.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            smallImageText: self.small_image_text.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            partyId: self.party_id.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            partySize: self.party_size as c_int,
            partyMax: self.party_max as c_int,
            matchSecret: self.match_secret.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            joinSecret: self.join_secret.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            spectateSecret: self.spectate_secret.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
            instance: self.instance as int8_t,
        }
    }
}