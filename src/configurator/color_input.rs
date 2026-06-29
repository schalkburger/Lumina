use freya::prelude::*;

use crate::util::colors::{parse_hex, TRANSPARENT};

#[derive(PartialEq)]
pub struct ColorInputControl {
  initial: Option<String>,
  on_change: EventHandler<String>,
}

impl ColorInputControl {
  pub fn new(initial: Option<String>, on_change: EventHandler<String>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for ColorInputControl {
  fn render(&self) -> impl IntoElement {
    let id = use_a11y();
    let focus_status = use_focus(id);
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial.clone().unwrap_or_default());

    let swatch_color = parse_hex(&value.read()).unwrap_or(TRANSPARENT);

    use_side_effect(move || {
      if !focus_status.read().is_focused() {
        on_change.call(value.read().clone());
      }
    });

    rect()
      .direction(Direction::Horizontal)
      .cross_align(Alignment::Center)
      .child(
        rect()
          .width(Size::px(20.))
          .height(Size::px(20.))
          .corner_radius(CornerRadius::new_all(4.))
          .background(swatch_color)
          .margin(Gaps::new(0., 6., 0., 0.)),
      )
      .child(Input::new(value).a11y_id(id).width(Size::px(100.)))
  }
}
