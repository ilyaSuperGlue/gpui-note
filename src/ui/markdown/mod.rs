use gpui_kit::{
    BorrowAppContext, ClickEvent, Context, IntoElement, ParentElement, Render, StyleRefinement,
    Styled, Task, Window,
    component::{WindowExt, clipboard::Clipboard, text::TextView},
    div, rems,
};
use std::time::Duration;

use crate::{AppState, NoteMode};

pub struct MarkdownViewer {
    active: Option<u64>,
    source: String,
    prefix_len: usize,
    visible: bool,
    update_task: Option<Task<()>>,
}

impl MarkdownViewer {
    pub fn new(ctx: &mut Context<Self>, visible_in: NoteMode) -> Self {
        let (active, source, prefix_len) = Self::source(ctx.global::<AppState>());

        let visible = ctx.global::<AppState>().mode == visible_in;
        ctx.observe_global::<AppState>(move |this, cx| {
            let state = cx.global::<AppState>();
            let became_visible = state.mode == visible_in && !this.visible;
            this.visible = state.mode == visible_in;
            if !this.visible {
                return;
            }

            let (active, source, prefix_len) = Self::source(state);
            if became_visible || active != this.active {
                this.active = active;
                this.source = source;
                this.prefix_len = prefix_len;
                this.update_task = None;
                cx.notify();
            } else if source != this.source {
                this.debounce_update(cx);
            }
        })
        .detach();

        Self {
            active,
            source,
            prefix_len,
            visible,
            update_task: None,
        }
    }

    fn source(state: &AppState) -> (Option<u64>, String, usize) {
        let active = state.active;
        let note = active.and_then(|id| state.notes.iter().find(|note| note.id == id));
        let prefix = note
            .map(|note| format!("# {}\n\n", note.title))
            .unwrap_or_default();
        let source = format!(
            "{prefix}{}",
            note.map(|note| note.text.as_str()).unwrap_or_default()
        );
        let prefix_len = prefix.len();
        (active, source, prefix_len)
    }

    fn debounce_update(&mut self, cx: &mut Context<Self>) {
        self.update_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(150))
                .await;
            let _ = this.update(cx, |this, cx| {
                let (active, source, prefix_len) = Self::source(cx.global::<AppState>());
                this.active = active;
                this.source = source;
                this.prefix_len = prefix_len;
                this.update_task = None;
                cx.notify();
            });
        }));
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let active = self.active;
        let source = self.source.clone();
        let prefix_len = self.prefix_len;

        div().size_full().child(
            TextView::markdown("markdown-preview", source)
                .on_link_click(|url, event, _window, cx| {
                    if event.is_right_click() {
                        println!("Show a context menu for {url}");
                        return;
                    }

                    match event {
                        ClickEvent::Mouse(click) if click.up.modifiers.control => {
                            println!("Open {url} in an internal view");
                        }
                        _ => cx.open_url(url),
                    }
                })
                .on_task_toggle(move |event, _window, cx| {
                    let Some(id) = active else { return };
                    let range = event.marker_range();
                    if range.start < prefix_len {
                        return;
                    }
                    let (start, end) = (range.start - prefix_len, range.end - prefix_len);
                    let marker = if event.checked() { "[x]" } else { "[ ]" };

                    cx.update_global::<AppState, _>(|state, _| {
                        if let Some(note) = state.notes.iter_mut().find(|n| n.id == id) {
                            if end <= note.text.len()
                                && note.text.is_char_boundary(start)
                                && note.text.is_char_boundary(end)
                            {
                                note.text.replace_range(start..end, marker);
                            }
                        }
                    });
                })
                .code_block_actions(|code_block, _window, _cx| {
                    let code = code_block.code().to_string();
                    gpui_kit::div().child(Clipboard::new("copy-code").value(code).on_copied(
                        |_, window, cx| {
                            window.push_notification(format!("code Copied"), cx);
                        },
                    ))
                })
                .inline_code_style(
                    StyleRefinement::default()
                        .text_size(rems(1.))
                        .px(rems(0.25)),
                )
                .scrollable(true),
        )
    }
}
