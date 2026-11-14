use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;
use reqwest_client::ReqwestClient;
use std::sync::Arc;
mod store;
mod ui;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteMode {
    Edit,
    Preview,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Hello,
    Note,
}

pub struct Navigator {
    hello: Entity<ui::HelloWorld>,
    note: Entity<ui::Note>,
    page: Page,
}

impl Navigator {
    pub fn new(cx: &mut Context<Self>, win: &mut Window) -> Self {
        let hello = cx.new(|_| ui::HelloWorld::new());
        let note = cx.new(|_cx| ui::Note::new(win, _cx));

        cx.observe_global::<AppState>(|this, cx| {
            let page = cx.global::<AppState>().page;
            if page != this.page {
                this.page = page;
                cx.notify();
            }
        })
        .detach();
        Self {
            hello,
            note,
            page: Page::Hello,
        }
    }
}

impl Render for Navigator {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pages: Page = cx.global::<AppState>().page;
        let content = match pages {
            Page::Hello => div().size_full().child(self.hello.clone()),
            Page::Note => div().size_full().child(self.note.clone()),
        };

        div().size_full().child(content)
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Notes {
    pub id: u64,
    pub title: String,
    pub text: String,
}

pub struct AppState {
    pub page: Page,
    pub notes: Vec<Notes>,
    pub active: Option<u64>,
    pub next_id: u64,
    pub mode: NoteMode,
}

impl Global for AppState {}

fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        Theme::update(cx, |theme| {
            theme.background = rgb(0x131215).into();
        });
        cx.set_http_client(Arc::new(ReqwestClient::user_agent("note.rs").unwrap()));

        let notes = store::load();
        let next_id = notes
            .iter()
            .map(|note| note.id)
            .max()
            .map_or(0, |id| id.saturating_add(1));
        let active = notes.first().map(|note| note.id);

        cx.set_global(AppState {
            page: Page::Note,
            notes,
            next_id,
            active,
            mode: NoteMode::Edit,
        });

        open_window(WindowOptions::default(), cx, |win, cx| {
            cx.new(|_cx| Navigator::new(_cx, win))
        })
        .expect("Failed to open window");
    });
}
