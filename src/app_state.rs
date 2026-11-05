use std::{
  collections::HashMap,
  sync::{Arc, OnceLock, RwLock, atomic::AtomicBool},
  time::{Duration, Instant},
};

use crate::{
  config::{Config, TransportMode},
  payloads::{Notification, SoundboardSoundPayload},
  user::{PremiumType, User, UserVoiceState},
  util::bridge::BridgeMessage,
};

/// Thread-safe shared state for background threads.
pub type SharedAppState = Arc<RwLock<AppState>>;

/// Queue feeding the single thread that ends speaking grace periods.
static GRACE_TX: OnceLock<flume::Sender<(String, Instant)>> = OnceLock::new();

fn grace_sender(
  shared: &SharedAppState,
  redraw_tx: &flume::Sender<()>,
) -> &'static flume::Sender<(String, Instant)> {
  GRACE_TX.get_or_init(|| {
    let (tx, rx) = flume::unbounded::<(String, Instant)>();
    let shared = shared.clone();
    let redraw_tx = redraw_tx.clone();

    // One long-lived thread ends every grace period. Spawning a thread per period made memory
    // grow on Windows with the OpenGL renderer. Deadlines arrive in order since they all use the
    // same grace length, so waiting on each one in turn is enough.
    std::thread::spawn(move || {
      for (user_id, until) in rx.iter() {
        if let Some(wait) = until.checked_duration_since(Instant::now()) {
          std::thread::sleep(wait);
        }
        end_grace(&shared, &redraw_tx, &user_id, until);
      }
    });

    tx
  })
}

/// Keep a user shown as speaking for `grace_ms` after they stop, so short pauses
/// between words do not flicker the row.
pub fn start_grace(
  shared: &SharedAppState,
  redraw_tx: &flume::Sender<()>,
  user: &mut User,
  grace_ms: u32,
) {
  if grace_ms == 0 {
    user.voice_state = UserVoiceState::NotSpeaking;
    return;
  }
  if user.grace_until.is_some() {
    return;
  }

  let until = Instant::now() + Duration::from_millis(u64::from(grace_ms));
  user.grace_until = Some(until);
  let _ = grace_sender(shared, redraw_tx).send((user.id.clone(), until));
}

fn end_grace(
  shared: &SharedAppState,
  redraw_tx: &flume::Sender<()>,
  user_id: &str,
  until: Instant,
) {
  let mut state = shared.write().unwrap();
  let Some(user) = state
    .voice_users
    .iter_mut()
    .find(|user| user.id == user_id && user.grace_until == Some(until))
  else {
    return;
  };

  user.grace_until = None;
  if user.voice_state == UserVoiceState::Speaking {
    user.voice_state = UserVoiceState::NotSpeaking;
  }
  let _ = redraw_tx.send(());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn grace_ends_speaking_after_deadline() {
    let shared: SharedAppState = Arc::new(RwLock::new(AppState::new()));
    let (redraw_tx, _redraw) = flume::unbounded();

    {
      let mut user = User {
        name: "user".into(),
        id: "1".into(),
        avatar: String::new(),
        voice_state: UserVoiceState::Speaking,
        streaming: false,
        camera: false,
        volume: 100.,
        grace_until: None,
      };
      start_grace(&shared, &redraw_tx, &mut user, 10);
      assert_eq!(user.voice_state, UserVoiceState::Speaking);
      shared.write().unwrap().voice_users.push(user);
    }

    std::thread::sleep(Duration::from_millis(150));

    let state = shared.read().unwrap();
    assert_eq!(
      state.voice_users[0].voice_state,
      UserVoiceState::NotSpeaking
    );
    assert!(state.voice_users[0].grace_until.is_none());
  }
}

#[derive(Debug, Clone)]
pub struct AppState {
  pub config: Config,
  pub user_id: String,
  pub premium_type: PremiumType,
  pub transport_mode: TransportMode,
  pub current_channel: String,
  pub current_guild_id: String,
  pub is_open: bool,
  pub is_censor: bool, // Used in modded clients but not IPC
  pub voice_users: Vec<User>,
  pub messages: Vec<Notification>,
  pub soundboard_cache: HashMap<String, Vec<SoundboardSoundPayload>>,

  // Name caches
  pub guild_names: HashMap<String, String>,
  pub channel_names: HashMap<String, String>,

  pub ws_sender: Option<flume::Sender<BridgeMessage>>,

  pub recording_keybind: Arc<AtomicBool>,
}

impl Default for AppState {
  fn default() -> Self {
    Self::new()
  }
}

impl AppState {
  pub fn new() -> Self {
    let mut default_guild_names: HashMap<String, String> = HashMap::new();
    default_guild_names.insert("0".into(), "Default".into());

    Self {
      config: Config::default(),
      user_id: String::new(),
      premium_type: PremiumType::None,
      transport_mode: TransportMode::Ipc,
      current_channel: String::new(),
      current_guild_id: String::new(),
      is_open: false,
      is_censor: false,
      voice_users: vec![],
      messages: vec![],
      soundboard_cache: HashMap::new(),

      guild_names: default_guild_names,
      channel_names: HashMap::new(),

      ws_sender: None,
      recording_keybind: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn send(&mut self, message: BridgeMessage) {
    if let Some(sender) = &self.ws_sender {
      sender.send(message).unwrap_or_default();
    }
  }

  pub fn notify(&mut self, notification: Notification) {
    let messages_len = self.messages.len();

    // Keep the last 3 elements
    if messages_len > 3 {
      self.messages.drain(0..messages_len - 3);
    }

    self.messages.push(notification);
  }
}
