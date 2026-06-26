#[derive(Debug, Clone, Default)]
pub struct WindowProperties {
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub modal: bool,
    pub has_parent: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WindowRule {
    app_id: Option<String>,
    title: Option<String>,
    floating: Option<bool>,
}

impl WindowRule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = Some(floating);
        self
    }

    pub fn matches(&self, properties: &WindowProperties) -> bool {
        self.app_id
            .as_deref()
            .is_none_or(|app_id| properties.app_id.as_deref() == Some(app_id))
            && self
                .title
                .as_deref()
                .is_none_or(|title| properties.title.as_deref() == Some(title))
    }

    pub fn floating_action(&self) -> Option<bool> {
        self.floating
    }
}
