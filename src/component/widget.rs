use super::color::Color;

#[derive(Debug, Clone)]
pub struct Block {
    color: Color,
    text: String,
    icon: String,
}

impl Block {
    /// Builder chain for component.
    pub fn new<T: Into<String>, E: Into<String>>(icon: T, text: E) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
            color: Color::new(),
        }
    }

    pub fn text_fg<T: Into<String>>(mut self, fg: T) -> Self {
        self.color = self.color.text_fg(fg);
        self
    }

    pub fn text_color<T: Into<String>>(mut self, fg: T, bg: T) -> Self {
        self.color = self.color.text(fg, bg);
        self
    }

    pub fn icon_color<T: Into<String>>(mut self, fg: T, bg: T) -> Self {
        self.color = self.color.icon(fg, bg);
        self
    }

    pub fn icon_fg<T: Into<String>>(mut self, fg: T) -> Self {
        self.color = self.color.icon_fg(fg);
        self
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::new();

        let fg = self.color.fg.as_ref().unwrap_or(&s);
        let bg = self.color.bg.as_ref().unwrap_or(&s);
        let icon_fg = self.color.icon_fg.as_ref().unwrap_or(&s);
        let icon_bg = self.color.icon_bg.as_ref().unwrap_or(&s);
        // [icon] [text]
        write!(
            f,
            "{}{}{} {}{}{}",
            icon_fg, icon_bg, self.icon, fg, bg, self.text
        )
    }
}
