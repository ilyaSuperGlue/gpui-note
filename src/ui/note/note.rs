use gpui_kit::{
    base::{
        dock::{DockArea, DockLayout, DockPlacement},
        h_flex,
    },
    component::{
        ActiveTheme, IconName, Theme, ThemeMode,
        button::{Button, ButtonVariants},
        dock::{DockSkin, panel_handle},
    },
    *,
};

use crate::{
    AppState, NoteMode,
    ui::note::{editor::NoteEditor, list::NoteList, preview::NotePreview},
};

pub struct Note {
    dock_area: Entity<DockArea>,
    note_preview: Entity<NotePreview>,
}

impl Note {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let note_list = cx.new(|ctx| NoteList::new(window, ctx));
        let note_editor = cx.new(|ctx| NoteEditor::new(window, ctx));
        let note_preview = cx.new(|ctx| NotePreview::new(ctx));

        let left_layout = DockLayout::tabs().panel_view(panel_handle(note_list.clone()), cx);
        let center_layout = DockLayout::tabs().panel_view(panel_handle(note_editor.clone()), cx);
        let dock_area = cx.new(|cx| {
            DockArea::new("notes-dock", Some(1), window, cx).with_renderer(DockSkin::new(cx))
        });

        dock_area.update(cx, |dock, cx| {
            dock.set_center(center_layout, window, cx);

            dock.set_dock(DockPlacement::Left, left_layout, window, cx);
            dock.set_dock_size(DockPlacement::Left, px(280.), window, cx);
        });

        let mut this = Self {
            dock_area,
            note_preview,
        };

        this.sync_preview_dock(cx.global::<AppState>().mode, window, cx);

        cx.observe_global_in::<AppState>(window, |this, window, cx| {
            this.sync_preview_dock(cx.global::<AppState>().mode, window, cx);
        })
        .detach();

        this
    }
}

impl Note {
    fn sync_preview_dock(&mut self, mode: NoteMode, window: &mut Window, cx: &mut Context<Self>) {
        let has_right_dock = self.dock_area.read(cx).has_dock(DockPlacement::Right);

        match (mode, has_right_dock) {
            (NoteMode::Edit, false) => {
                let layout =
                    DockLayout::tabs().panel_view(panel_handle(self.note_preview.clone()), cx);

                self.dock_area.update(cx, |dock, cx| {
                    dock.set_dock(DockPlacement::Right, layout, window, cx);
                    dock.set_dock_size(DockPlacement::Right, px(420.), window, cx);
                });
            }
            (NoteMode::Preview, true) => {
                self.dock_area.update(cx, |dock, cx| {
                    dock.remove_dock(DockPlacement::Right, window, cx);
                });
            }
            _ => {}
        }
    }
}

impl Render for Note {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = Theme::global(cx).mode.is_dark();
        let theme_icon = if is_dark {
            IconName::Sun
        } else {
            IconName::Moon
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                h_flex()
                    .w_full()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .self_start()
                    .p_3()
                    .gap_2()
                    .child(
                        Button::new("toggle-theme")
                            .icon(theme_icon)
                            .ghost()
                            .on_click(cx.listener(|_, _, _, cx| {
                                let next_mode = if Theme::global(cx).mode.is_dark() {
                                    ThemeMode::Light
                                } else {
                                    ThemeMode::Dark
                                };
                                Theme::change(next_mode, None, cx);
                                if next_mode == ThemeMode::Dark {
                                    Theme::update(cx, |theme| {
                                        theme.background = rgb(0x131215).into();
                                    });
                                }
                            })),
                    )
                    .child("Note.rs")
                    .font_weight(FontWeight(600.0)),
            )
            .child(self.dock_area.clone())
    }
}
