use embassy_time::{Duration, Instant};
use heapless::{Vec, String};
use embassy_rp::rtc::{DateTime, DayOfWeek};
use core::fmt::Write;

const LETTERS: [char; 45] = [' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '$', '&', '#', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', '.', '-', '?', '!'];
const LETTER_COUNT: u8 = LETTERS.len() as u8;

use crate::{MESSAGE_LENGTH, HOUR_SETTING};

enum Mode {
    Hour,
    Messages
}
enum HourMode {
    Mode24,
    Mode12
}

pub struct Settings {
    mode: Mode,
    reset_frequency: Option<Duration>,
    message_change_frequency: Option<Duration>,
    messages: Vec<[u8; MESSAGE_LENGTH], 10>,
    current_message_id: Option<usize>,
    current_time: DateTime,
    hour_mode: HourMode,
}

pub enum TimeSetting {
    Hour(u8),
    Minute(u8),
    Second(u8)
}

#[derive(Clone, Copy)]
pub enum HomePosition {
    Mode,
    ResetFrequency,
    ResetFrequencyAdjust,
    HourSettings,
    MessageChangeFrequency,
    MessageChangeFrequencyAdjust,
    Messages,
    CurrentMessage,
    CurrentMessageAdjust,
    DisplayOff,
    DisplayCurrentOff
}
#[derive(Clone, Copy)]
pub enum HourSettingsPosition {
    Hour,
    HourAdjust(u8),
    Minute,
    MinuteAdjust(u8),
    Second,
    TwelveOrTwentyFour
}
#[derive(Clone, Copy)]
pub enum MessagesPosition {
    AddMessage,
    Message(usize)
}
#[derive(Clone, Copy)]
pub enum MessagePosition {
    EditMessage,
    Message(usize), // letter position
    Delete
}

#[derive(Clone, Copy)]
pub enum Screens {
    Home(HomePosition),
    HourSettings(HourSettingsPosition),
    Messages(MessagesPosition),
    Message(usize, MessagePosition),
}

#[derive(Clone, Copy)]
pub struct Disp (Screens);

impl Disp {
    pub const fn new() -> Self {
        Self(Screens::Home(HomePosition::Mode))
    }
    pub fn screen(&self) -> Screens {
        self.0
    }

    pub fn up(&mut self, settings: &mut Settings) {
        self.0 = self.0.up(settings);
    }

    pub fn down(&mut self, settings: &mut Settings) {
        self.0 = self.0.down(settings);
    }

    pub async fn enter(&mut self, settings: &mut Settings) {
        self.0 = self.0.enter(settings).await;
    }

    pub fn back(&mut self) {
        self.0 = self.0.back();
    }

    pub fn output(&self, settings: &Settings) -> [String<20>; 4] {
        let mut line0 = String::new();
        let mut line1 = String::new();
        let mut line2 = String::new();
        let mut line3 = String::new();

        match self.0 {
            Screens::Home(position) => match position {
                HomePosition::Mode => {
                    let _ = write!(line0, "> {}", HomePosition::Mode.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line2, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                },
                HomePosition::ResetFrequency => {
                    let _ = write!(line0, "  {}", HomePosition::Mode.get_line(settings));
                    let _ = write!(line1, "> {}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line2, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                },
                HomePosition::ResetFrequencyAdjust => {
                    let _ = write!(line0, "  {}", HomePosition::Mode.get_line(settings));
                    let _ = write!(line1, ">>{}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line2, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                },
                HomePosition::HourSettings => {
                    let _ = write!(line0, "  {}", HomePosition::Mode.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line2, "> {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                },
                HomePosition::MessageChangeFrequency => {
                    let _ = write!(line0, "  {}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line2, "> {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::Messages.get_line(settings));
                },
                HomePosition::MessageChangeFrequencyAdjust => {
                    let _ = write!(line0, "  {}", HomePosition::ResetFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line2, ">>{}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::Messages.get_line(settings));
                },
                HomePosition::Messages => {
                    let _ = write!(line0, "  {}", HomePosition::HourSettings.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line2, "> {}", HomePosition::Messages.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::CurrentMessage.get_line(settings));
                },
                HomePosition::CurrentMessage => {
                    let _ = write!(line0, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::Messages.get_line(settings));
                    let _ = write!(line2, "> {}", HomePosition::CurrentMessage.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::DisplayOff.get_line(settings));
                },
                HomePosition::CurrentMessageAdjust => {
                    let _ = write!(line0, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::Messages.get_line(settings));
                    let _ = write!(line2, ">>{}", HomePosition::CurrentMessage.get_line(settings));
                    let _ = write!(line3, "  {}", HomePosition::DisplayOff.get_line(settings));
                },
                HomePosition::DisplayOff => {
                    let _ = write!(line0, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::Messages.get_line(settings));
                    let _ = write!(line2, "  {}", HomePosition::CurrentMessage.get_line(settings));
                    let _ = write!(line3, "> {}", HomePosition::DisplayOff.get_line(settings));
                },
                HomePosition::DisplayCurrentOff => {
                    let _ = write!(line0, "  {}", HomePosition::MessageChangeFrequency.get_line(settings));
                    let _ = write!(line1, "  {}", HomePosition::Messages.get_line(settings));
                    let _ = write!(line2, "  {}", HomePosition::CurrentMessage.get_line(settings));
                    let _ = write!(line3, ">>{}", HomePosition::DisplayOff.get_line(settings));
                },
            },
            Screens::HourSettings(position) => match position {
                HourSettingsPosition::Hour => {
                    let _ = write!(line0, "> {}", HourSettingsPosition::Hour.get_line(settings));
                    let _ = write!(line1, "  {}", HourSettingsPosition::Minute.get_line(settings));
                    let _ = write!(line2, "  {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "  {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
                HourSettingsPosition::HourAdjust(_) => {
                    let _ = write!(line0, ">>{}", position.get_line(settings));
                    let _ = write!(line1, "  {}", HourSettingsPosition::Minute.get_line(settings));
                    let _ = write!(line2, "  {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "  {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
                HourSettingsPosition::Minute => {
                    let _ = write!(line0, "  {}", HourSettingsPosition::Hour.get_line(settings));
                    let _ = write!(line1, "> {}", HourSettingsPosition::Minute.get_line(settings));
                    let _ = write!(line2, "  {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "  {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
                HourSettingsPosition::MinuteAdjust(_) => {
                    let _ = write!(line0, "  {}", HourSettingsPosition::Hour.get_line(settings));
                    let _ = write!(line1, ">>{}", position.get_line(settings));
                    let _ = write!(line2, "  {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "  {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
                HourSettingsPosition::Second => {
                    let _ = write!(line0, "  {}", HourSettingsPosition::Hour.get_line(settings));
                    let _ = write!(line1, "  {}", HourSettingsPosition::Minute.get_line(settings));
                    let _ = write!(line2, "> {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "  {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
                HourSettingsPosition::TwelveOrTwentyFour => {
                    let _ = write!(line0, "  {}", HourSettingsPosition::Hour.get_line(settings));
                    let _ = write!(line1, "  {}", HourSettingsPosition::Minute.get_line(settings));
                    let _ = write!(line2, "  {}", HourSettingsPosition::Second.get_line(settings));
                    let _ = write!(line3, "> {}", HourSettingsPosition::TwelveOrTwentyFour.get_line(settings));
                },
            },
            Screens::Messages(position) => match position {
                MessagesPosition::AddMessage => {
                    let _ = write!(line0, "> {}", MessagesPosition::AddMessage.get_line(settings));

                    let messages_len = settings.messages.len();
                    if messages_len > 0 {
                        let _ = write!(line1, "  {}", MessagesPosition::Message(0).get_line(settings));
                    }
                    if messages_len > 1 {
                        let _ = write!(line2, "  {}", MessagesPosition::Message(1).get_line(settings));
                    }
                    if messages_len > 2 {
                        let _ = write!(line3, "  {}", MessagesPosition::Message(2).get_line(settings));
                    }
                },
                MessagesPosition::Message(0) => {
                    let _ = write!(line0, "  {}", MessagesPosition::AddMessage.get_line(settings));
                    let _ = write!(line1, "> {}", MessagesPosition::Message(0).get_line(settings));
                    let messages_len = settings.messages.len();
                    if messages_len > 1 {
                        let _ = write!(line2, "  {}", MessagesPosition::Message(1).get_line(settings));
                    }
                    if messages_len > 2 {
                        let _ = write!(line3, "  {}", MessagesPosition::Message(2).get_line(settings));
                    }
                }
                MessagesPosition::Message(message_id) => {
                    let messages_len = settings.messages.len();

                    let _ = write!(line0, "  {}", MessagesPosition::Message(message_id - 1).get_line(settings));
                    let _ = write!(line1, "> {}", MessagesPosition::Message(message_id).get_line(settings));
                    if messages_len > message_id + 1 {
                        let _ = write!(line2, "  {}", MessagesPosition::Message(message_id + 1).get_line(settings));
                    }
                    if messages_len > message_id + 2 {
                        let _ = write!(line3, "  {}", MessagesPosition::Message(message_id + 2).get_line(settings));
                    }
                }
            },
            Screens::Message(message_id, position) => match position {
                MessagePosition::EditMessage => {
                    let _ = write!(line0, "  {}", MessagePosition::Message(0).get_line(settings, message_id));
                    let _ = write!(line1, "> {}", MessagePosition::EditMessage.get_line(settings, message_id));
                    let _ = write!(line2, "  {}", MessagePosition::Delete.get_line(settings, message_id));},
                MessagePosition::Message(letter_id) => {
                    let _ = write!(line0, "> {}", MessagePosition::Message(letter_id).get_line(settings, message_id));
                    let _ = write!(line1, "  {}", MessagePosition::Message(letter_id).get_marker(settings));
                },
                MessagePosition::Delete => {
                    let _ = write!(line0, "  {}", MessagePosition::Message(0).get_line(settings, message_id));
                    let _ = write!(line1, "  {}", MessagePosition::EditMessage.get_line(settings, message_id));
                    let _ = write!(line2, "> {}", MessagePosition::Delete.get_line(settings, message_id));}
            }
        };

        [line0, line1, line2, line3]
    }
}

impl Screens {
    fn up(self, settings: &mut Settings) -> Self {
        match self {
            Self::Home(position) => match position {
                HomePosition::Mode => self,
                HomePosition::ResetFrequency => Self::Home(HomePosition::Mode),
                HomePosition::ResetFrequencyAdjust => {
                    settings.decrease_reset_frequency();
                    self
                },
                HomePosition::HourSettings => Self::Home(HomePosition::ResetFrequency),
                HomePosition::MessageChangeFrequency => Self::Home(HomePosition::HourSettings),
                HomePosition::MessageChangeFrequencyAdjust => {
                    settings.decrease_message_change_frequency();
                    self
                },
                HomePosition::Messages => Self::Home(HomePosition::MessageChangeFrequency),
                HomePosition::CurrentMessage => Self::Home(HomePosition::Messages),
                HomePosition::CurrentMessageAdjust => {
                    settings.decrease_current_message_id();
                    self
                },
                HomePosition::DisplayOff => Self::Home(HomePosition::CurrentMessage),
                HomePosition::DisplayCurrentOff => Self::Home(HomePosition::DisplayOff),
            },
            Self::HourSettings(position) => match position {
                HourSettingsPosition::Hour => self,
                HourSettingsPosition::HourAdjust(hour) => {
                    if hour == 0 {
                        Self::HourSettings(HourSettingsPosition::HourAdjust(23))
                    } else {
                        Self::HourSettings(HourSettingsPosition::HourAdjust(hour - 1))
                    }
                },
                HourSettingsPosition::Minute => Self::HourSettings(HourSettingsPosition::Hour),
                HourSettingsPosition::MinuteAdjust(minute) => {
                    if minute == 0 {
                        Self::HourSettings(HourSettingsPosition::MinuteAdjust(59))
                    } else {
                        Self::HourSettings(HourSettingsPosition::MinuteAdjust(minute - 1))
                    }
                },
                HourSettingsPosition::Second => Self::HourSettings(HourSettingsPosition::Minute),
                HourSettingsPosition::TwelveOrTwentyFour => Self::HourSettings(HourSettingsPosition::Second)
            },
            Self::Messages(position) => match position {
                MessagesPosition::AddMessage => self,
                MessagesPosition::Message(msg_id) => {
                    if msg_id == 0 {
                        Self::Messages(MessagesPosition::AddMessage)
                    } else {
                        Self::Messages(MessagesPosition::Message(msg_id-1))
                    }
                }
            },
            Self::Message(message_id, position) => match position {
                MessagePosition::EditMessage => self,
                MessagePosition::Message(letter_index) => {
                    settings.decrease_letter(message_id, letter_index);
                    self
                },
                MessagePosition::Delete => Self::Message(message_id, MessagePosition::EditMessage)
            }
        }
    }

    fn down(self, settings: &mut Settings) -> Self {
        match self {
            Self::Home(position) => match position {
                HomePosition::Mode => Self::Home(HomePosition::ResetFrequency),
                HomePosition::ResetFrequency => Self::Home(HomePosition::HourSettings),
                HomePosition::ResetFrequencyAdjust => {
                    settings.increase_reset_frequency();
                    self
                },
                HomePosition::HourSettings => Self::Home(HomePosition::MessageChangeFrequency),
                HomePosition::MessageChangeFrequency => Self::Home(HomePosition::Messages),
                HomePosition::MessageChangeFrequencyAdjust => {
                    settings.increase_message_change_frequency();
                    self
                },
                HomePosition::Messages => Self::Home(HomePosition::CurrentMessage),
                HomePosition::CurrentMessage => Self::Home(HomePosition::DisplayOff),
                HomePosition::CurrentMessageAdjust => {
                    settings.increase_current_message_id();
                    self
                },
                HomePosition::DisplayOff => self,
                HomePosition::DisplayCurrentOff => Self::Home(HomePosition::DisplayOff),
            },
            Self::HourSettings(position) => match position {
                HourSettingsPosition::Hour => Self::HourSettings(HourSettingsPosition::Minute),
                HourSettingsPosition::HourAdjust(hour) => {
                    if hour == 23 {
                        Self::HourSettings(HourSettingsPosition::HourAdjust(0))
                    } else {
                        Self::HourSettings(HourSettingsPosition::HourAdjust(hour + 1))
                    }
                },
                HourSettingsPosition::Minute => Self::HourSettings(HourSettingsPosition::Second),
                HourSettingsPosition::MinuteAdjust(minute) => {
                    if minute == 59 {
                        Self::HourSettings(HourSettingsPosition::MinuteAdjust(0))
                    } else {
                        Self::HourSettings(HourSettingsPosition::MinuteAdjust(minute + 1))
                    }
                },
                HourSettingsPosition::Second => Self::HourSettings(HourSettingsPosition::TwelveOrTwentyFour),
                HourSettingsPosition::TwelveOrTwentyFour => self
            },
            Self::Messages(position) => match position {
                MessagesPosition::AddMessage => {
                    if !settings.messages.is_empty() {
                        Self::Messages(MessagesPosition::Message(0))
                    } else {
                        self
                    }
                },
                MessagesPosition::Message(msg_id) => {
                    if msg_id != settings.messages.len() - 1 {
                        Self::Messages(MessagesPosition::Message(msg_id + 1))
                    } else {
                        self
                    }
                }
            },
            Self::Message(message_id, position) => match position {
                MessagePosition::EditMessage => Self::Message(message_id, MessagePosition::Delete),
                MessagePosition::Message(letter_index) => {
                    settings.increase_letter(message_id, letter_index);
                    self
                },
                MessagePosition::Delete => self
            }
        }
    }

    async fn enter(self, settings: &mut Settings) -> Self {
        match self {
            Self::Home(position) => match position {
                HomePosition::Mode => {
                    settings.toggle_mode();
                    self
                },
                HomePosition::ResetFrequency => Self::Home(HomePosition::ResetFrequencyAdjust),
                HomePosition::ResetFrequencyAdjust => Self::Home(HomePosition::ResetFrequency),
                HomePosition::HourSettings => Self::HourSettings(HourSettingsPosition::Hour),
                HomePosition::MessageChangeFrequency => Self::Home(HomePosition::MessageChangeFrequencyAdjust),
                HomePosition::MessageChangeFrequencyAdjust => Self::Home(HomePosition::MessageChangeFrequency),
                HomePosition::Messages => Self::Messages(MessagesPosition::AddMessage),
                HomePosition::CurrentMessage => Self::Home(HomePosition::CurrentMessageAdjust),
                HomePosition::CurrentMessageAdjust => Self::Home(HomePosition::CurrentMessage),
                HomePosition::DisplayOff => Self::Home(HomePosition::DisplayCurrentOff),
                HomePosition::DisplayCurrentOff => Self::Home(HomePosition::DisplayOff),
            },
            Self::HourSettings(position) => match position {
                HourSettingsPosition::Hour => Self::HourSettings(HourSettingsPosition::HourAdjust(settings.current_time.hour)),
                HourSettingsPosition::HourAdjust(hour) => {
                    HOUR_SETTING.send(TimeSetting::Hour(hour)).await;
                    Self::HourSettings(HourSettingsPosition::Hour)
                },
                HourSettingsPosition::Minute => Self::HourSettings(HourSettingsPosition::MinuteAdjust(settings.current_time.minute)),
                HourSettingsPosition::MinuteAdjust(minute) => {
                    HOUR_SETTING.send(TimeSetting::Minute(minute)).await;
                    Self::HourSettings(HourSettingsPosition::Minute)
                },
                HourSettingsPosition::Second => {
                    HOUR_SETTING.send(TimeSetting::Second(0)).await;
                    self
                },
                HourSettingsPosition::TwelveOrTwentyFour => {
                    settings.toggle_hour_mode();
                    self
                }
            },
            Self::Messages(position) => match position {
                MessagesPosition::AddMessage => {
                    if let Some(index) = settings.add_message() {
                        Self::Message(index, MessagePosition::Message(0))
                    } else {
                        self
                    }
                },
                MessagesPosition::Message(msg_id) => {
                    Self::Message(msg_id, MessagePosition::EditMessage)
                }
            },
            Self::Message(message_id, position) => match position {
                MessagePosition::EditMessage => Self::Message(message_id, MessagePosition::Message(0)),
                MessagePosition::Message(letter_index) => {
                    if letter_index == MESSAGE_LENGTH - 1 {
                        Self::Message(message_id, MessagePosition::Message(0))
                    } else {
                        Self::Message(message_id, MessagePosition::Message(letter_index + 1))
                    }
                },
                MessagePosition::Delete => {
                    settings.delete_message(message_id);
                    Self::Messages(MessagesPosition::AddMessage)
                }
            }
        }
    }

    fn back(self) -> Self {
        match self {
            Self::Home(position) => match position {
                HomePosition::Mode => self,
                HomePosition::ResetFrequency => self,
                HomePosition::ResetFrequencyAdjust => Self::Home(HomePosition::ResetFrequency),
                HomePosition::HourSettings => self,
                HomePosition::MessageChangeFrequency => self,
                HomePosition::MessageChangeFrequencyAdjust => Self::Home(HomePosition::MessageChangeFrequency),
                HomePosition::Messages => self,
                HomePosition::CurrentMessage => self,
                HomePosition::CurrentMessageAdjust => Self::Home(HomePosition::CurrentMessage),
                HomePosition::DisplayOff => self,
                HomePosition::DisplayCurrentOff => Self::Home(HomePosition::DisplayOff),
            },
            Self::HourSettings(position) => match position {
                HourSettingsPosition::Hour => Self::Home(HomePosition::HourSettings),
                HourSettingsPosition::HourAdjust(_) => Self::HourSettings(HourSettingsPosition::Hour),
                HourSettingsPosition::Minute => Self::Home(HomePosition::HourSettings),
                HourSettingsPosition::MinuteAdjust(_) => Self::HourSettings(HourSettingsPosition::Minute),
                HourSettingsPosition::Second => Self::Home(HomePosition::HourSettings),
                HourSettingsPosition::TwelveOrTwentyFour => Self::Home(HomePosition::HourSettings)
            },
            Self::Messages(position) => match position {
                MessagesPosition::AddMessage => Self::Home(HomePosition::Messages),
                MessagesPosition::Message(_) => Self::Home(HomePosition::Messages)
            },
            Self::Message(message_id, position) => match position {
                MessagePosition::EditMessage => Self::Messages(MessagesPosition::Message(message_id)),
                MessagePosition::Message(_) => Self::Message(message_id, MessagePosition::EditMessage),
                MessagePosition::Delete => Self::Messages(MessagesPosition::Message(message_id))
            }
        }
    }
}


impl HourMode {
    fn convert(&self, hour: u8) -> u8 {
        match self {
            Self::Mode24 => hour,
            Self::Mode12 => {
                match hour {
                    0 => 12,
                    1..=12 => hour,
                    13..=23 => hour - 12,
                    _ => 0
                }
            }
        }
    }
}

impl Settings {
    pub const fn new() -> Self {
        Self { mode: Mode::Messages, reset_frequency: Some(Duration::from_secs(600)), message_change_frequency: None, messages: Vec::new(), current_message_id: None, current_time: DateTime { year: 2023, month: 9, day: 2, day_of_week: DayOfWeek::Saturday, hour: 15, minute: 38, second: 0 }, hour_mode: HourMode::Mode24 }
    }

    pub fn get_message(&self) -> [u8; MESSAGE_LENGTH] {
        match self.mode {
            Mode::Hour => {
                let mut buf = [0; MESSAGE_LENGTH];
                let hours = self.hour_mode.convert(self.current_time.hour);
                let minute = self.current_time.minute;

                let decades = hours / 10;
                if decades != 0 {
                    buf[0] = decades + 30;
                }
                buf[1] = (hours % 10) + 30;
                buf[2] = 40; // index of `:`
                buf[3] = (minute / 10) + 30;
                buf[4] = (minute % 10) + 30;
                buf
            },
            Mode::Messages => {
                if let Some(message_id) = self.current_message_id {
                    self.messages[message_id]
                } else {
                    // no messages
                    [0; MESSAGE_LENGTH]
                }
            }
        }
    }
    /// Switches current message if in message mode and message frequency elapsed
    /// Returns true if message was switched
    /// Returns None if not in message mode or there are no messages
    pub fn switch_message(&mut self, last_switch: Instant) -> Option<bool> {
        match self.mode {
            Mode::Messages => Some({
                let now = Instant::now();
                if now.duration_since(last_switch) > self.message_change_frequency? {
                    if self.current_message_id? == self.messages.len() - 1 {
                        self.current_message_id = Some(0);
                    } else {
                        self.current_message_id = Some(self.current_message_id? + 1);
                    }
                    true
                } else {
                    false
                }
            }),
            Mode::Hour => None
        }
    }
    pub fn should_reset(&self, last_reset: Instant) -> bool {
        if let Some(reset_frequency) = self.reset_frequency {
            let now = Instant::now();
            now.duration_since(last_reset) > reset_frequency
        } else {
            false
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Hour => Mode::Messages,
            Mode::Messages => Mode::Hour
        }
    }
    fn toggle_hour_mode(&mut self) {
        self.hour_mode = match self.hour_mode {
            HourMode::Mode24 => HourMode::Mode12,
            HourMode::Mode12 => HourMode::Mode24
        }
    }

    fn increase_reset_frequency(&mut self) {
        match self.reset_frequency {
            Some(reset_frequency) => {
                let new = reset_frequency.checked_add(Duration::from_secs(60));
                if new.is_some() { // on overflow, leave as is.
                    self.reset_frequency = new;
                }
            },
            None => self.reset_frequency = Some(Duration::from_secs(60))
        }
    }
    fn decrease_reset_frequency(&mut self) {
        if let Some(reset_frequency) = self.reset_frequency {
            self.reset_frequency = reset_frequency.checked_sub(Duration::from_secs(60));  // on underflow, set to None
        }
    }

    fn increase_message_change_frequency(&mut self) {
        match self.message_change_frequency {
            Some(message_change_frequency) => {
                let new = message_change_frequency.checked_add(Duration::from_secs(10));
                if new.is_some() { // on overflow, leave as is.
                    self.message_change_frequency = new;
                }
            },
            None => self.message_change_frequency = Some(Duration::from_secs(10))
        }
    }
    fn decrease_message_change_frequency(&mut self) {
        if let Some(message_change_frequency) = self.message_change_frequency {
            self.message_change_frequency = message_change_frequency.checked_sub(Duration::from_secs(10));  // on underflow, set to None
        }
    }

    fn increase_current_message_id(&mut self) {
        if let Some(current_id) = self.current_message_id {
            if current_id < self.messages.len() - 1 {
                self.current_message_id = Some(current_id + 1);
            }
        }
    }
    fn decrease_current_message_id(&mut self) {
        if let Some(current_id) = self.current_message_id {
            if current_id != 0 {
                self.current_message_id = Some(current_id - 1);
            }
        }
    }

    fn increase_letter(&mut self, message_id: usize, letter_id: usize) {
        if let Some(message) = self.messages.get_mut(message_id) {
            if message[letter_id] == LETTER_COUNT - 1 {
                message[letter_id] = 0;
            } else {
                message[letter_id] += 1;
            }
        };
    }
    fn decrease_letter(&mut self, message_id: usize, letter_id: usize) {
        if let Some(message) = self.messages.get_mut(message_id) {
            if message[letter_id] == 0 {
                message[letter_id] = LETTER_COUNT - 1;
            } else {
                message[letter_id] -= 1;
            }
        };
    }

    fn add_message(&mut self) -> Option<usize> {
        self.messages.push([0; MESSAGE_LENGTH]).ok()?;
        if self.current_message_id.is_none() {
            self.current_message_id = Some(0);
        };
        Some(self.messages.len() - 1)
    }
    fn delete_message(&mut self, message_id: usize) {
        self.messages.remove(message_id);
        if self.messages.is_empty() {
            self.current_message_id = None;
        }
        if let Some(current_id) = self.current_message_id {
            if current_id == self.messages.len() {
                self.current_message_id = Some(current_id - 1);
            }
        }
    }

    fn message_as_text(&self, message_id: usize) -> Option<String<MESSAGE_LENGTH>> {
        let mut buf = String::new();
        for letter_id in self.messages[message_id] {
            let _ = buf.push(LETTERS[letter_id as usize]);
        }
        Some(buf)}
    fn current_message(&self) -> Option<String<MESSAGE_LENGTH>> {
        self.message_as_text(self.current_message_id?)
    }

    pub fn current_time(&self) -> &DateTime { &self.current_time }
    /// Set the current time inm the settings
    /// Returns true if the value changed
    pub fn set_current_time(&mut self, time: DateTime) -> bool {
        if self.current_time.hour != time.hour || self.current_time.minute != time.minute || self.current_time.second != time.second {
            self.current_time = time;
            true
        } else {
            false
        }
    }
}

impl HomePosition {
    fn get_line(&self, settings: &Settings) -> String<18> {
        let mut buf = String::new();
        match self {
            Self::Mode => {
                let mode = match settings.mode {
                    Mode::Hour => "Horloge",
                    Mode::Messages => "Messages"
                };
                write!(&mut buf, "Mode: {}", mode).unwrap();
            },
            Self::ResetFrequency | Self::ResetFrequencyAdjust => {
                let rst_freq_min = settings.reset_frequency.map_or(0, |v| v.as_secs()) / 60;
                write!(&mut buf, "Freq Reset: {}m", rst_freq_min).unwrap();
            },
            Self::HourSettings => {
                buf = String::from("Reglages Horloge");
            },
            Self::MessageChangeFrequency | Self::MessageChangeFrequencyAdjust => {
                let frequency = settings.message_change_frequency.map_or(0, |v| v.as_secs());
                let min = frequency / 60;
                let sec = frequency % 60;
                write!(&mut buf, "Freq Msg: {}m {}s", min, sec).unwrap();
            },
            Self::Messages => {
                buf = String::from("Messages");
            },
            Self::CurrentMessage | Self::CurrentMessageAdjust => {
                let msg = settings.current_message();
                write!(&mut buf, "Msg: {}", msg.unwrap_or("".into())).unwrap();
            },
            Self::DisplayOff | Self::DisplayCurrentOff => {
                buf = String::from("Eteindre LCD");
            }
        };
        buf
    }
}

impl HourSettingsPosition {
    fn get_line(&self, settings: &Settings) -> String<18> {
        let mut buf = String::new();
        match self {
            Self::Hour => {
                write!(&mut buf, "Heures: {}h", settings.current_time.hour).unwrap();
            },
            Self::HourAdjust(val) => {
                write!(&mut buf, "Heures: {}h", val).unwrap();
            },
            Self::Minute => {
                write!(&mut buf, "Minutes: {}m", settings.current_time.minute).unwrap();
            },
            Self::MinuteAdjust(val) => {
                write!(&mut buf, "Minutes: {}m", val).unwrap();
            },
            Self::Second => {
                write!(&mut buf, "Secondes: {}s", settings.current_time.second).unwrap();
            },
            Self::TwelveOrTwentyFour => {
                let mode = match settings.hour_mode {
                    HourMode::Mode24 => "24h",
                    HourMode::Mode12 => "12h"
                };
                write!(&mut buf, "12h/24h: {}", mode).unwrap();
            }
        };
        buf
    }
}

impl MessagesPosition {
    fn get_line(&self, settings: &Settings) -> String<18> {
        let mut buf = String::new();
        match self {
            Self::AddMessage => {
                buf = "Nouveau Message".into();
            },
            Self::Message(message_id) => {
                write!(&mut buf, "[{}]", settings.message_as_text(*message_id).unwrap_or("".into())).unwrap();
            }
        };
        buf
    }
}

impl MessagePosition {
    fn get_line(&self, settings: &Settings, message_id: usize) -> String<18> {
        let mut buf = String::new();
        match self {
            Self::EditMessage => {
                buf = "Modifier".into();
            },
            Self::Message(_letter_id) => {
                write!(&mut buf, "[{}]", settings.message_as_text(message_id).unwrap_or("".into())).unwrap();
            },
            Self::Delete => {
                buf = "Supprimer".into();
            }
        };
        buf
    }
    fn get_marker(&self, _settings: &Settings) -> String<18> {
        let mut buf = String::new();
        if let Self::Message(letter_id) = self {
            let _ = buf.push(' ');
            for _ in 0..*letter_id {
                let _ = buf.push(' ');
            }
            let _ = buf.push('^');
        };
        buf
    }
}
