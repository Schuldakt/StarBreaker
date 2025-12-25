//! UI theme configuration

/// UI Theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Theme::Dark
    }
    
    /// Create light theme
    pub fn light() -> Self {
        Theme::Light
    }
    
    /// Check if dark theme
    pub fn is_dark(&self) -> bool {
        matches!(self, Theme::Dark)
    }
    
    /// Check if light theme
    pub fn is_light(&self) -> bool {
        matches!(self, Theme::Light)
    }
    
    /// Toggle between dark and light
    pub fn toggle(&mut self) {
        *self = match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}
