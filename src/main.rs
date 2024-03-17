use iced::widget::{button, column, text};
use iced::{Application, Command, Element, Settings, Theme};

#[derive(Default)]
struct Counter {
    value: i32,
}

impl Application for Counter {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn view(&self) -> Element<Message> {
        // We use a column: a simple vertical layout
        column![
            // The increment button. We tell it to produce an
            // `Increment` message when pressed
            button("+").on_press(Message::Increment),

            // We show the value of the counter here
            text(self.value).size(50),

            // The decrement button. We tell it to produce a
            // `Decrement` message when pressed
            button("-").on_press(Message::Decrement),
        ].into()
    }

    fn new(_flags: ()) -> (Counter, Command<Self::Message>) {
        (Counter::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("A simple counter")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
        Command::none()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Increment,
    Decrement,
}

fn main() {
    Counter::run(Settings::default()).expect("run failed");
}
