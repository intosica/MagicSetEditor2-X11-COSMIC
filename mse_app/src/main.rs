//! mse_app – libcosmic front-end. Usage: `mse [path/to/set.mse-set]`

use std::path::PathBuf;

use cosmic::{
    app::{Core, Settings, Task},
    iced::Length,
    widget, Application, Element,
};
use mse_core::{render::render_card, Card, Set, Template};

const APP_ID: &str = "io.github.intosica.MagicSetEditorX11Cosmic";

fn main() -> cosmic::iced::Result {
    tracing_init();
    let path = std::env::args_os().nth(1).map(PathBuf::from);
    cosmic::app::run::<App>(Settings::default(), path)
}

fn tracing_init() { /* TODO: tracing-subscriber */ }

struct App {
    core: Core,
    set: Set,
    template: Template,
    selected: usize,
    preview: Option<widget::image::Handle>,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectCard(usize),
    NewCard,
    DeleteCard,
    FieldChanged(&'static str, String),
    OpenSet,
    ExportPng,
}

/// Fields shown in the form: (card field key, label).
const FORM_FIELDS: &[(&str, &str)] = &[
    ("name", "Name"),
    ("casting_cost", "Mana cost"),
    ("type_line", "Type line"),
    ("rule_text", "Rules text"),
    ("pt", "Power / Toughness"),
    ("image", "Art (path)"),
];

impl App {
    fn current(&self) -> Option<&Card> {
        self.set.cards.get(self.selected)
    }

    fn refresh_preview(&mut self) {
        // Render first (immutable borrow ends here), then touch self.status.
        let result = self.set.cards.get(self.selected).map(|c| render_card(c, &self.template));
        self.preview = match result {
            Some(Ok(r)) => Some(widget::image::Handle::from_rgba(r.width, r.height, r.rgba)),
            Some(Err(e)) => {
                self.status = format!("render failed: {e}");
                None
            }
            None => None,
        };
    }
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = Option<PathBuf>;
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Message>) {
        let mut status = String::from("No set loaded");
        let mut set = Set::default();
        if let Some(p) = flags {
            match Set::load(&p) {
                Ok(s) => {
                    status = format!("Loaded {} ({} cards)", p.display(), s.cards.len());
                    set = s;
                }
                Err(e) => status = format!("Failed to load {}: {e}", p.display()),
            }
        }
        if set.cards.is_empty() {
            let mut c = Card::new();
            c.set("name", "New Card");
            set.cards.push(c);
        }
        let mut app = App { core, set, template: Template::builtin_magic_placeholder(), selected: 0, preview: None, status };
        app.refresh_preview();
        (app, Task::none())
    }

    /// Top bar. TODO: replace with cosmic::widget::menu::bar (File/Edit/Export/Settings).
    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![
            widget::button::text("Open…").on_press(Message::OpenSet).into(),
            widget::button::text("New card").on_press(Message::NewCard).into(),
            widget::button::text("Delete card").on_press(Message::DeleteCard).into(),
            widget::button::text("Export PNG").on_press(Message::ExportPng).into(),
        ]
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectCard(i) => {
                self.selected = i;
                self.refresh_preview();
            }
            Message::NewCard => {
                let mut c = Card::new();
                c.set("name", "New Card");
                self.set.cards.push(c);
                self.selected = self.set.cards.len() - 1;
                self.refresh_preview();
            }
            Message::DeleteCard => {
                if self.selected < self.set.cards.len() {
                    self.set.cards.remove(self.selected);
                    self.selected = self.selected.saturating_sub(1);
                    self.refresh_preview();
                }
            }
            Message::FieldChanged(key, value) => {
                if let Some(c) = self.set.cards.get_mut(self.selected) {
                    c.set(key, value);
                }
                self.refresh_preview();
            }
            Message::OpenSet => self.status = "TODO: file dialog (rfd / xdg-portal)".into(),
            Message::ExportPng => {
                if let Some(c) = self.current() {
                    let res = render_card(c, &self.template).and_then(|r| r.save_png("card.png"));
                    self.status = match res {
                        Ok(()) => "Exported card.png".into(),
                        Err(e) => format!("export failed: {e}"),
                    };
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Left: card list
        let mut items: Vec<Element<Message>> = Vec::new();
        for (i, c) in self.set.cards.iter().enumerate() {
            let label = c.display_name().to_owned();
            let btn = if i == self.selected {
                widget::button::suggested(label)
            } else {
                widget::button::standard(label)
            };
            items.push(btn.on_press(Message::SelectCard(i)).width(Length::Fill).into());
        }
        let list = widget::column(items).spacing(4);
        let left = widget::container(widget::scrollable(list).height(Length::Fill))
            .width(Length::Fixed(260.0))
            .height(Length::Fill)
            .padding(8);

        // Right: form + preview
        let mut fields: Vec<Element<Message>> = Vec::new();
        if let Some(card) = self.current() {
            for &(key, label) in FORM_FIELDS {
                fields.push(
                    widget::text_input(label, card.get(key).to_owned())
                        .label(label)
                        .on_input(move |v| Message::FieldChanged(key, v))
                        .into(),
                );
            }
        }
        let form = widget::column(fields).spacing(8);
        let preview: Element<Message> = match &self.preview {
            Some(h) => widget::image(h.clone()).into(),
            None => widget::text("No preview").into(),
        };
        let right = widget::row(vec![
            widget::container(form).width(Length::FillPortion(1)).padding(8).into(),
            widget::container(preview).width(Length::FillPortion(1)).padding(8).into(),
        ])
        .spacing(16);

        let body = widget::row(vec![left.into(), right.into()]).height(Length::Fill);
        widget::column(vec![
            body.into(),
            widget::container(widget::text(self.status.clone())).padding(6).into(),
        ])
        .into()
    }
}
