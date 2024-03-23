use iced::executor;
use iced::highlighter::{self, Highlighter};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text,
    text_editor, tooltip,
};
use iced::{
    Alignment, Application, Command, Element, Font, Length, Settings,
    Subscription,
};
use iced_aw::{TabBar, TabLabel};

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        ..Settings::default()
    })
}

struct Editor {
    theme: highlighter::Theme,
    fragment_index: usize,
    fragments: Vec<FragmentContent>,
}

struct FragmentContent {
    file: Option<PathBuf>,
    content: text_editor::Content,
    is_loading: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
    TabSelected(usize),
    TabClosed(usize),
    TabNew,
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let fragment_content = FragmentContent {
            file: None,
            content: text_editor::Content::new(),
            is_loading: true,
            is_dirty: false,
        };
        (
            Self {
                theme: highlighter::Theme::SolarizedDark,
                fragment_index: 0,
                fragments: vec![fragment_content],
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("Editor - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let fragment = &mut self.fragments[self.fragment_index];
        match message {
            Message::ActionPerformed(action) => {
                fragment.is_dirty = fragment.is_dirty || action.is_edit();

                fragment.content.perform(action);

                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
            Message::NewFile => {
                if !fragment.is_loading {
                    fragment.file = None;
                    fragment.content = text_editor::Content::new();
                }

                Command::none()
            }
            Message::OpenFile => {
                if fragment.is_loading {
                    Command::none()
                } else {
                    fragment.is_loading = true;

                    Command::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                fragment.is_loading = false;
                fragment.is_dirty = false;

                if let Ok((path, contents)) = result {
                    fragment.file = Some(path);
                    fragment.content = text_editor::Content::with_text(&contents);
                }

                Command::none()
            }
            Message::SaveFile => {
                if fragment.is_loading {
                    Command::none()
                } else {
                    fragment.is_loading = true;

                    Command::perform(
                        save_file(fragment.file.clone(), fragment.content.text()),
                        Message::FileSaved,
                    )
                }
            }
            Message::FileSaved(result) => {
                fragment.is_loading = false;

                if let Ok(path) = result {
                    fragment.file = Some(path);
                    fragment.is_dirty = false;
                }

                Command::none()
            }
            Message::TabSelected(index) => {
                Command::none()
            }
            Message::TabClosed(index) => {
                Command::none()
            }
            Message::TabNew => {
                let fragment_content = FragmentContent {
                    file: None,
                    content: text_editor::Content::new(),
                    is_loading: true,
                    is_dirty: false,
                };
                self.fragments.push(fragment_content);
                self.fragment_index = self.fragments.len() - 1;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => {
                Some(Message::SaveFile)
            }
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        let idx = self.fragment_index;
        let controls = row![
            action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.fragments[idx].is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.fragments[idx].is_dirty.then_some(Message::SaveFile)
            ),
            action(
                new_tab_icon(),
                "New Tab",
                Some(Message::TabNew)
            ),
            horizontal_space(),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let status = row![
            text(if let Some(path) = &self.fragments[idx].file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.fragments[idx].content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        let tabs = self
            .fragments
            .iter()
            .fold(
                TabBar::new(Message::TabSelected),
                |tab_bar, fragment| {
                    let label = if let Some(file) = &fragment.file {
                        TabLabel::Text(file.display().to_string())
                    } else {
                        TabLabel::Text(String::from("New"))
                    };
                    let idx = tab_bar.size();
                    tab_bar.push(idx, label)
                },
            )
            .on_close(Message::TabClosed)
            .tab_width(Length::Shrink)
            .spacing(5.0)
            .padding(5.0)
            .text_size(32.0);

        column![
            controls,
            tabs,
            text_editor(&self.fragments[idx].content)
                .height(Length::Fill)
                .on_action(Message::ActionPerformed)
                .highlight::<Highlighter>(
                    highlighter::Settings {
                        theme: self.theme,
                        extension: self
                            .fragments[idx]
                            .file
                            .as_deref()
                            .and_then(Path::extension)
                            .and_then(ffi::OsStr::to_str)
                            .map(str::to_string)
                            .unwrap_or(String::from("rs")),
                    },
                    |highlight, _theme| highlight.to_format()
                ),
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(picked_file.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

async fn save_file(
    path: Option<PathBuf>,
    contents: String,
) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).width(30).center_x());

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box)
        .into()
    } else {
        action.style(theme::Button::Secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn new_tab_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    text(codepoint).into()
}
