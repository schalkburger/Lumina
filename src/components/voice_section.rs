use freya::prelude::*;

use crate::{
  app_state::{AppState, SharedAppState},
  components::{UserRow, VoiceControls, voice_controls::RedrawSender},
  config::{AxisAlignment, CornerAlignment, DisplayVoiceMembers},
  user::{User, UserVoiceState},
  util::text::censor,
};

pub struct VoiceSection {
  pub voice_users: Vec<User>,
  pub current_user: Option<User>,
  pub is_open: bool,
  pub is_censor: bool,
  pub user_alignment: String,
  pub user_offset_x: i32,
  pub user_offset_y: i32,
  pub display_voice_members: DisplayVoiceMembers,
  pub user_row_background: Option<String>,
  pub app_state: State<AppState>,
  pub soundboard_open: State<bool>,
  pub shared: SharedAppState,
  pub redraw_tx: RedrawSender,
}

impl PartialEq for VoiceSection {
  fn eq(&self, other: &Self) -> bool {
    self.voice_users == other.voice_users
      && self.current_user == other.current_user
      && self.is_open == other.is_open
      && self.is_censor == other.is_censor
      && self.user_alignment == other.user_alignment
      && self.user_offset_x == other.user_offset_x
      && self.user_offset_y == other.user_offset_y
      && self.display_voice_members == other.display_voice_members
      && self.user_row_background == other.user_row_background
  }
}

impl Component for VoiceSection {
  fn render(&self) -> impl IntoElement {
    let alignment = CornerAlignment::from_str(&self.user_alignment);
    let gaps = alignment.to_gaps(self.user_offset_x, self.user_offset_y);
    let is_right_aligned = alignment.x == AxisAlignment::End;

    let mut sorted_users = self.voice_users.clone();
    sorted_users.sort_by(|a, b| a.id.cmp(&b.id));

    let filtered_users: Vec<_> = sorted_users
      .into_iter()
      .filter(|user| match self.display_voice_members {
        DisplayVoiceMembers::Always => true,
        DisplayVoiceMembers::AlwaysSemiTransparent => true,
        DisplayVoiceMembers::WhenSpeaking => {
          user.voice_state == UserVoiceState::Speaking || self.is_open
        }
      })
      .collect();

    let has_users = !filtered_users.is_empty();
    let shared = self.shared.clone();
    let redraw_tx = self.redraw_tx.clone();

    let base = rect()
      .direction(Direction::Vertical)
      .cross_align(alignment.x.to_freya())
      .main_align(alignment.y.to_freya())
      .position(Position::new_absolute().top(0.).left(0.))
      .background(Color::TRANSPARENT)
      .height(Size::fill())
      .width(Size::fill())
      .padding(gaps);

    let with_users = filtered_users.iter().fold(base, |el, user| {
      let mut u = user.clone();
      if self.is_censor {
        u.name = censor(&u.name);
      }
      el.child(UserRow {
        user: u,
        is_open: self.is_open,
        is_right_aligned,
        is_voice_semitransparent: matches!(
          self.display_voice_members,
          DisplayVoiceMembers::AlwaysSemiTransparent
        ),
        background: self.user_row_background.clone(),
      })
    });

    if has_users {
      with_users.maybe_child(self.current_user.clone().map(|user| {
        VoiceControls {
          user,
          app_state: self.app_state,
          soundboard_open: self.soundboard_open,
          shared,
          redraw_tx,
        }
      }))
    } else {
      with_users
    }
  }
}
