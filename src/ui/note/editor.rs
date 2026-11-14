use gpui_kit::{
    base::{
        dock::{Panel as BasePanel, *},
        h_flex,
        input::{EditorState, InputEvent, InputState},
        v_flex,
    },
    component::{
        Disableable, Icon, IconName, WindowExt,
        button::{Button, ButtonVariants},
        clipboard::Clipboard,
        dock::Panel,
        empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
        input::{Editor, Input},
    },
    *,
};
use std::time::Duration;

use crate::{AppState, NoteMode, store, ui::markdown::MarkdownViewer};

pub struct NoteEditor {
    title: Entity<InputState>,
    text: Entity<EditorState>,
    focus_handle: FocusHandle,
    active: Option<u64>,
    mode: NoteMode,
    save_task: Option<Task<()>>,
    markdown_viewer: Entity<MarkdownViewer>,
}

impl NoteEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let markdown_viewer = cx.new(|ctx| MarkdownViewer::new(ctx, NoteMode::Preview));

        let (active, mode, initial_title, initial_text) = {
            let state = cx.global::<AppState>();
            let active = state.active;
            let mode = state.mode;
            let note = active.and_then(|id| state.notes.iter().find(|note| note.id == id));

            (
                active,
                mode,
                note.map(|note| note.title.clone()).unwrap_or_default(),
                note.map(|note| note.text.clone()).unwrap_or_default(),
            )
        };

        let title = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_title)
                .placeholder("title")
        });
        let text = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .folding(true)
                .searchable(true)
                .scroll_bounce(true)
                .default_value(initial_text)
                .placeholder("put your text here")
                .autocorrect(true)
        });

        cx.observe_global_in::<AppState>(window, |this, window, cx| {
            let state = cx.global::<AppState>();
            let new_id = state.active;
            let new_mode = state.mode;
            let active_changed = new_id != this.active;
            let mode_changed = new_mode != this.mode;
            this.mode = new_mode;
            if !active_changed && !mode_changed {
                return;
            }
            if !active_changed {
                cx.notify();
                return;
            }
            this.active = new_id;

            let (title, text) = new_id
                .and_then(|id| {
                    cx.global::<AppState>()
                        .notes
                        .iter()
                        .find(|note| note.id == id)
                })
                .map(|note| (note.title.clone(), note.text.clone()))
                .unwrap_or_default();

            this.title
                .update(cx, |i, cx| i.set_value(title, window, cx));
            this.text.update(cx, |e, cx| e.set_value(text, window, cx));
            cx.notify();
        })
        .detach();
        cx.subscribe_in(
            &title,
            window,
            |this, state, event: &InputEvent, _, cx| match event {
                InputEvent::Change => {
                    let value = state.read(cx).value().to_string();
                    this.write_back(Some(value), None, cx);
                }
                _ => {}
            },
        )
        .detach();

        cx.subscribe_in(
            &text,
            window,
            |this, state, event: &InputEvent, _, cx| match event {
                InputEvent::Change => {
                    let value = state.read(cx).value().to_string();
                    this.write_back(None, Some(value), cx);
                }
                _ => {}
            },
        )
        .detach();

        Self {
            title,
            text,
            focus_handle: cx.focus_handle(),
            active,
            mode,
            save_task: None,
            markdown_viewer,
        }
    }
}

impl NoteEditor {
    fn write_back(&mut self, title: Option<String>, text: Option<String>, cx: &mut Context<Self>) {
        let Some(id) = self.active else { return };

        let changed = {
            let state = cx.global::<AppState>();
            state
                .notes
                .iter()
                .find(|note| note.id == id)
                .is_some_and(|note| {
                    title.as_ref().is_some_and(|t| *t != note.title)
                        || text.as_ref().is_some_and(|t| *t != note.text)
                })
        };
        if !changed {
            return;
        }

        cx.update_global::<AppState, _>(|state, _| {
            if let Some(note) = state.notes.iter_mut().find(|note| note.id == id) {
                if let Some(t) = title {
                    note.title = t;
                }
                if let Some(t) = text {
                    note.text = t;
                }
            }
        });
        self.schedule_save(cx);
    }

    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        self.save_task = Some(cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            let notes = cx.update(|cx| cx.global::<AppState>().notes.clone());
            cx.background_spawn(async move {
                if let Err(e) = store::save(&notes) {
                    eprintln!("save failed: {e}");
                }
            })
            .await;
        }));
    }
}

impl EventEmitter<PanelEvent> for NoteEditor {}

impl Focusable for NoteEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl BasePanel for NoteEditor {
    fn panel_name(&self) -> &'static str {
        "editor"
    }
}

impl Panel for NoteEditor {
    fn title(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .child(
                Button::new("edit-mode")
                    .icon(IconName::FileText)
                    .ghost()
                    .disabled(self.mode == NoteMode::Edit)
                    .on_click(ctx.listener(|_, _, _, cx| {
                        cx.update_global::<AppState, _>(|state, _| {
                            state.mode = NoteMode::Edit;
                        });
                    })),
            )
            .child(
                Button::new("preview-mode")
                    .icon(IconName::Eye)
                    .ghost()
                    .disabled(self.mode == NoteMode::Preview)
                    .on_click(ctx.listener(|_, _, _, cx| {
                        cx.update_global::<AppState, _>(|state, _| {
                            state.mode = NoteMode::Preview;
                        });
                    })),
            )
            .child(
                Clipboard::new("copy-code")
                    .value(self.text.read(ctx).value())
                    .on_copied(|_, window, cx| {
                        window.push_notification(format!("Text Copied"), cx);
                    }),
            )
    }

    fn title_suffix(
        &mut self,
        _: &mut Window,
        ctx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        Some(
            div().child(
                Button::new("delete")
                    .icon(IconName::Ban)
                    .danger()
                    .disabled(self.active.is_none())
                    .on_click(ctx.listener(|_, _, window, cx| {
                        let Some(id) = cx.global::<AppState>().active else {
                            return;
                        };

                        window.open_alert_dialog(cx, move |alert, _, _| {
                            alert
                                .title("Delete this note?")
                                .description("This note will be permanently deleted.")
                                .confirm()
                                .ok_text("Confirm")
                                .cancel_text("Cancel")
                                .on_ok(move |_, _, cx| {
                                    cx.update_global::<AppState, _>(|state, _| {
                                        if let Some(index) =
                                            state.notes.iter().position(|note| note.id == id)
                                        {
                                            state.notes.remove(index);
                                            state.active = state
                                                .notes
                                                .get(index)
                                                .or_else(|| state.notes.last())
                                                .map(|note| note.id);
                                        }
                                    });

                                    let notes = cx.global::<AppState>().notes.clone();
                                    cx.background_spawn(async move {
                                        if let Err(error) = store::save(&notes) {
                                            eprintln!("save failed: {error}");
                                        }
                                    })
                                    .detach();

                                    true
                                })
                        });
                    })),
            ),
        )
    }
}

impl Render for NoteEditor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let content = if self.mode == NoteMode::Edit || self.active.is_none() {
            if self.active.is_some() {
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .p_3()
                    .gap_5()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(Input::new(&self.title).flex_1().px_3().py_4()),
                    )
                    .child(v_flex().flex_1().child(Editor::new(&self.text).h_full()))
                    .h_full()
                    .into_any_element()
            } else {
                v_flex()
                    .h_full()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(
                        Empty::new().header(
                            EmptyHeader::new()
                                .media(
                                    EmptyMedia::new()
                                        .with_variant(EmptyMediaVariant::Icon)
                                        .child(Icon::new(IconName::FileText)),
                                )
                                .title(EmptyTitle::new().child("No Notes yet"))
                                .description(
                                    EmptyDescription::new()
                                        .child("Create your first note to get started."),
                                ),
                        ),
                    )
                    .into_any_element()
            }
        } else {
            v_flex()
                .size_full()
                .min_w_0()
                .min_h_0()
                .p_4()
                .child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .child(self.markdown_viewer.clone()),
                )
                .into_any_element()
        };

        div().size_full().child(content)
    }
}
