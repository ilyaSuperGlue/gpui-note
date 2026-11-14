use std::rc::Rc;

use gpui_kit::{
    base::{
        VirtualListScrollHandle,
        dock::{Panel as BasePanel, *},
        h_flex, v_flex, v_virtual_list,
    },
    component::{IconName, Sizable, button::Button, dock::Panel},
    *,
};

use crate::{AppState, Notes};

pub struct NoteList {
    count: usize,
    scroll_handle: VirtualListScrollHandle,
    focus_handle: FocusHandle,
    item_sizes: Rc<Vec<Size<Pixels>>>,
}
fn sizes(count: usize) -> Rc<Vec<Size<Pixels>>> {
    Rc::new((0..count).map(|_| size(px(200.), px(70.))).collect())
}

impl NoteList {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let count = cx.global::<AppState>().notes.len();

        cx.observe_global::<AppState>(|this, cx| {
            let count = cx.global::<AppState>().notes.len();
            if count != this.count {
                this.count = count;
                this.item_sizes = sizes(count);
            }
            cx.notify();
        })
        .detach();

        Self {
            count,
            item_sizes: sizes(count),
            scroll_handle: VirtualListScrollHandle::new(),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for NoteList {}

impl Focusable for NoteList {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl BasePanel for NoteList {
    fn panel_name(&self) -> &'static str {
        "list"
    }
}

impl Panel for NoteList {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ""
    }

    fn toolbar_buttons(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> Option<Vec<Button>> {
        let mut buttons: Vec<Button> = Vec::new();
        buttons.push(
            Button::new("add")
                .icon(IconName::Plus)
                .large()
                .on_click(ctx.listener(|_, _, _, ctx| {
                    ctx.update_global::<AppState, _>(|state, _| {
                        let title = "new note".to_string();
                        let text = String::new();
                        let id = state.next_id;
                        state.next_id += 1;
                        state.notes.insert(0, Notes { id, title, text });
                        state.active = Some(id);
                    })
                })),
        );
        Some(buttons)
    }
}

impl Render for NoteList {
    fn render(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        h_flex().flex_col().h_full().w_full().child(
            v_virtual_list(
                ctx.entity().clone(),
                "note-list",
                self.item_sizes.clone(),
                move |_view, visible_range, _window, ctx| {
                    let (rows, active): (Vec<_>, _) = {
                        let state = ctx.global::<AppState>();
                        let rows = visible_range
                            .filter_map(|ix| state.notes.get(ix))
                            .map(|n| {
                                let lines: Vec<String> = n
                                    .text
                                    .lines()
                                    .take(2)
                                    .map(|l| l.chars().take(80).collect())
                                    .collect();
                                (n.id, n.title.clone(), lines)
                            })
                            .collect();
                        (rows, state.active)
                    };

                    rows.into_iter()
                        .map(|(id, title, lines)| {
                            let selected = active == Some(id);
                            v_flex()
                                .id(("note", id))
                                .h(px(70.))
                                .w_full()
                                .px_3()
                                .py_2()
                                .border_1()
                                .rounded_md()
                                .border_color(if selected {
                                    rgb(0x3D9BDA)
                                } else {
                                    rgb(0x444444)
                                })
                                .on_click(ctx.listener(move |_, _, _, cx| {
                                    cx.update_global::<AppState, _>(|s, _| s.active = Some(id));
                                }))
                                .hover(|item| item.border_color(rgb(0x0D47DA)))
                                .child(
                                    v_flex()
                                        .w_full()
                                        .text_size(px(14.))
                                        .line_clamp(1)
                                        .text_ellipsis()
                                        .child(format!("{}", title))
                                        .font_weight(FontWeight(600.0)),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .text_size(px(10.))
                                        .children(
                                            lines.into_iter().map(|l| div().truncate().child(l)),
                                        ),
                                )
                        })
                        .collect()
                },
            )
            .mt_3()
            .p_3()
            .gap_3()
            .track_scroll(&self.scroll_handle),
        )
    }
}
