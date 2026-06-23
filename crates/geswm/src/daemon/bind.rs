use std::collections::HashSet;

use smithay::{backend::input::Keycode, input::keyboard::ModifiersState};

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyModifiers: u32 {
        const Empty = 0;
        const Shift = 1 << 0;
        const Ctrl = 1 << 1;
        const Alt = 1 << 2;
        const Super = 1 << 3;
    }
}

impl From<&ModifiersState> for KeyModifiers {
    fn from(modifiers: &ModifiersState) -> Self {
        let mut key_modifiers = KeyModifiers::Empty;
        if modifiers.shift {
            key_modifiers |= KeyModifiers::Shift;
        }
        if modifiers.ctrl {
            key_modifiers |= KeyModifiers::Ctrl;
        }
        if modifiers.alt {
            key_modifiers |= KeyModifiers::Alt;
        }
        if modifiers.logo {
            key_modifiers |= KeyModifiers::Super;
        }
        key_modifiers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyBind {
    pub modifiers: KeyModifiers,
    pub key: Keycode,
}

impl std::fmt::Display for KeyBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut modifiers = vec![];
        if self.modifiers.contains(KeyModifiers::Shift) {
            modifiers.push("Shift");
        }
        if self.modifiers.contains(KeyModifiers::Ctrl) {
            modifiers.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::Alt) {
            modifiers.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::Super) {
            modifiers.push("Super");
        }
        if modifiers.is_empty() {
            return write!(f, "{:?}", self.key);
        } else {
            write!(f, "{}+{:?}", modifiers.join("+"), self.key)
        }
    }
}

impl KeyBind {
    pub fn new(key: Keycode) -> Self {
        Self {
            modifiers: KeyModifiers::Empty,
            key,
        }
    }

    pub fn with_super(self) -> Self {
        Self {
            modifiers: self.modifiers | KeyModifiers::Super,
            key: self.key,
        }
    }

    pub fn with_alt(self) -> Self {
        Self {
            modifiers: self.modifiers | KeyModifiers::Alt,
            key: self.key,
        }
    }

    pub fn with_ctrl(self) -> Self {
        Self {
            modifiers: self.modifiers | KeyModifiers::Ctrl,
            key: self.key,
        }
    }

    pub fn with_shift(self) -> Self {
        Self {
            modifiers: self.modifiers | KeyModifiers::Shift,
            key: self.key,
        }
    }
}

impl std::hash::Hash for KeyBind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.modifiers.bits().hash(state);
        self.key.hash(state);
    }
}
