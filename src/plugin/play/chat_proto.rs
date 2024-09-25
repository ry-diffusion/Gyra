use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ChatComponent {
    Text(String),
    Translate(TranslateObject),
    PlayerMessage(ChatObject),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatObject {
    #[serde(rename = "text", default)]
    text: String,
    #[serde(rename = "extra", default)]
    extra: Option<Vec<ExtraComponent>>,
    #[serde(rename = "color", default)]
    color: Option<ChatColor>,
    #[serde(rename = "bold", default)]
    bold: Option<bool>,
    #[serde(rename = "italic", default)]
    italic: Option<bool>,
    #[serde(rename = "underlined", default)]
    underlined: Option<bool>,
    #[serde(rename = "strikethrough", default)]
    strikethrough: Option<bool>,
    #[serde(rename = "obfuscated", default)]
    obfuscated: Option<bool>,
}

// We handle "extra" to allow both strings and ChatComponents
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ExtraComponent {
    Text(String),
    Component(ChatComponent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranslateObject {
    #[serde(rename = "translate")]
    translate: String,
    #[serde(rename = "with", default)]
    with: Vec<ChatComponent>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ChatColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl ChatObject {
    pub fn to_plain_text(&self) -> String {
        let mut result = self.text.clone();

        // Handle "extra" field which can contain either strings or ChatComponents
        if let Some(extras) = &self.extra {
            for extra in extras {
                match extra {
                    ExtraComponent::Text(text) => result.push_str(text),
                    ExtraComponent::Component(component) => match component {
                        ChatComponent::Text(text) => result.push_str(text),
                        ChatComponent::PlayerMessage(obj) => result.push_str(&obj.to_plain_text()),
                        ChatComponent::Translate(trans_obj) => {
                            result.push_str(&trans_obj.to_plain_text())
                        }
                    },
                }
            }
        }

        result
    }
}

impl TranslateObject {
    pub fn to_plain_text(&self) -> String {
        match self.translate.as_str() {
            // For server announcements
            "chat.type.announcement" => {
                if self.with.len() == 2 {
                    let announcer = self.extract_plain_text(0);
                    let message = self.extract_plain_text(1);
                    format!("[Server Announcement] {}: {}", announcer, message)
                } else {
                    "[Server Announcement]".to_string()
                }
            }
            // For player chat messages
            "chat.type.text" => {
                if self.with.len() == 2 {
                    let player_name = self.extract_plain_text(0);
                    let message = self.extract_plain_text(1);
                    format!("[{}]: {}", player_name, message)
                } else {
                    "[Player Message]".to_string()
                }
            }
            _ => self.translate.clone(),
        }
    }

    pub fn extract_plain_text(&self, index: usize) -> String {
        match &self.with.get(index) {
            Some(ChatComponent::Text(text)) => text.clone(),
            Some(ChatComponent::PlayerMessage(obj)) => obj.to_plain_text(),
            Some(ChatComponent::Translate(trans_obj)) => trans_obj.to_plain_text(),
            _ => String::new(),
        }
    }
}

impl ChatComponent {
    pub fn to_plain_text(&self) -> String {
        match self {
            ChatComponent::Text(text) => text.clone(),
            ChatComponent::PlayerMessage(obj) => obj.to_plain_text(),
            ChatComponent::Translate(obj) => obj.to_plain_text(),
        }
    }
}
