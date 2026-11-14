use gpui_kit::base::dock::PanelEvent;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::*;

use crate::{AppState, Page};

pub struct HelloWorld {
    counter: i64,
}

impl HelloWorld {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl EventEmitter<PanelEvent> for HelloWorld {}

impl Render for HelloWorld {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let counter = self.counter;
        div()
            .flex()
            .flex_col()
            .child(
                Button::new("go to setting")
                    .secondary()
                    .label("go to setting")
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.update_global::<AppState, _>(|state, _cx| {
                            state.page = Page::Note;
                        });
                    })),
            )
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .child(format!("Button clicked {}!", counter))
            .child(
                Button::new("hello")
                    .primary()
                    .label("Click me")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.counter += 1;
                        cx.notify();
                    })),
            )
    }
}
