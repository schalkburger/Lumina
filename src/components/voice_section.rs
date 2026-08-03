use freya::prelude::*;

use crate::{
  app_state::SharedAppState,
  components::UserRow,
  config::{AxisAlignment, CornerAlignment, DisplayVoiceMembers, TransportMode},
  user::{User, UserVoiceState},
  util::text::censor,
};

pub struct VoiceSection {
  pub voice_users: Vec<User>,
  pub is_open: bool,
  pub is_censor: bool,
  pub user_alignment: String,
  pub user_offset_x: i32,
  pub user_offset_y: i32,
  pub display_voice_members: DisplayVoiceMembers,
  pub user_row_background: Option<String>,
  pub shared: SharedAppState,
}

impl PartialEq for VoiceSection {
  fn eq(&self, other: &Self) -> bool {
    self.voice_users == other.voice_users
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

    let base = rect()
      .direction(Direction::Vertical)
      .cross_align(alignment.x.to_freya())
      .main_align(alignment.y.to_freya())
      .position(Position::new_absolute().top(0.).left(0.))
      .background(Color::TRANSPARENT)
      .height(Size::fill())
      .width(Size::fill())
      .padding(gaps);

    let x_mult: i32 = match alignment.x {
      AxisAlignment::Start => 1,
      AxisAlignment::End => -1,
      AxisAlignment::Center => 0,
    };
    let y_mult: i32 = match alignment.y {
      AxisAlignment::Start => 1,
      AxisAlignment::End => -1,
      AxisAlignment::Center => 0,
    };

    let shared = self.shared.clone();

    rect()
      .width(Size::fill())
      .height(Size::fill())
      .child(ContextMenuViewer::new())
      .child(
        filtered_users.iter().fold(base, |el, user| {
          let mut u = user.clone();
          if self.is_censor {
            u.name = censor(&u.name);
          }
          // TODO websocket cannot change user volume yet
          let can_context_menu = {
            let state = shared.read().unwrap();
            state.user_id != u.id && state.config.transport_mode == TransportMode::Ipc
          };
          el.child(UserRow {
            user: u,
            is_open: self.is_open,
            is_right_aligned,
            is_voice_semitransparent: matches!(
              self.display_voice_members,
              DisplayVoiceMembers::AlwaysSemiTransparent
            ),
            can_context_menu,
            background: self.user_row_background.clone(),
            shared: shared.clone(),
            x_mult,
            y_mult,
          })
        }),
      )
  }
}
