use smithay::input::keyboard::XkbConfig;

#[derive(Debug, Clone)]
pub struct KeyboardConfiguration<'a> {
    pub rules: &'a str,
    pub model: &'a str,
    pub layout: &'a str,
    pub variant: &'a str,
    pub options: Option<String>,
    pub repeat_delay: i32,
    pub repeat_rate: i32,
}

impl<'a> From<KeyboardConfiguration<'a>> for XkbConfig<'a> {
    fn from(val: KeyboardConfiguration<'a>) -> Self {
        XkbConfig {
            rules: val.rules,
            model: val.model,
            layout: val.layout,
            variant: val.variant,
            options: val.options,
        }
    }
}
