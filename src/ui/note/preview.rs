use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window,
    base::{
        dock::{Panel as BasePanel, PanelEvent},
        v_flex,
    },
    component::dock::Panel,
};

use crate::{NoteMode, ui::markdown::MarkdownViewer};

pub struct NotePreview {
    focus_handle: FocusHandle,
    markdown_viewer: Entity<MarkdownViewer>,
}

impl NotePreview {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let markdown_viewer = cx.new(|ctx| MarkdownViewer::new(ctx, NoteMode::Edit));
        Self {
            focus_handle: cx.focus_handle(),
            markdown_viewer,
        }
    }
}

impl EventEmitter<PanelEvent> for NotePreview {}

impl Focusable for NotePreview {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl BasePanel for NotePreview {
    fn panel_name(&self) -> &'static str {
        "preview"
    }
}

impl Panel for NotePreview {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Preview"
    }
}

impl Render for NotePreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_2()
            .child(self.markdown_viewer.clone())
    }
}
