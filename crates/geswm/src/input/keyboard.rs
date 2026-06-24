use smithay::input::keyboard::ModifiersState;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XkbKeyCode(pub u32);

impl From<smithay::input::keyboard::KeysymHandle<'_>> for XkbKeyCode {
    fn from(value: smithay::input::keyboard::KeysymHandle<'_>) -> Self {
        Self(value.raw_code().raw())
    }
}

#[allow(non_upper_case_globals)]
impl XkbKeyCode {
    pub const Escape: Self = Self(9);
    pub const Num1: Self = Self(10);
    pub const Num2: Self = Self(11);
    pub const Num3: Self = Self(12);
    pub const Num4: Self = Self(13);
    pub const Num5: Self = Self(14);
    pub const Num6: Self = Self(15);
    pub const Num7: Self = Self(16);
    pub const Num8: Self = Self(17);
    pub const Num9: Self = Self(18);
    pub const Num0: Self = Self(19);
    pub const Minus: Self = Self(20);
    pub const Equal: Self = Self(21);
    pub const Backspace: Self = Self(22);
    pub const Tab: Self = Self(23);
    pub const Q: Self = Self(24);
    pub const W: Self = Self(25);
    pub const E: Self = Self(26);
    pub const R: Self = Self(27);
    pub const T: Self = Self(28);
    pub const Y: Self = Self(29);
    pub const U: Self = Self(30);
    pub const I: Self = Self(31);
    pub const O: Self = Self(32);
    pub const P: Self = Self(33);
    pub const LeftBracket: Self = Self(34);
    pub const RightBracket: Self = Self(35);
    pub const Return: Self = Self(36);
    pub const CtrlLeft: Self = Self(37);
    pub const A: Self = Self(38);
    pub const S: Self = Self(39);
    pub const D: Self = Self(40);
    pub const F: Self = Self(41);
    pub const G: Self = Self(42);
    pub const H: Self = Self(43);
    pub const J: Self = Self(44);
    pub const K: Self = Self(45);
    pub const L: Self = Self(46);
    pub const Semicolon: Self = Self(47);
    pub const Apostrophe: Self = Self(48);
    pub const Grave: Self = Self(49);
    pub const ShiftLeft: Self = Self(50);
    pub const Backslash: Self = Self(51);
    pub const Z: Self = Self(52);
    pub const X: Self = Self(53);
    pub const C: Self = Self(54);
    pub const V: Self = Self(55);
    pub const B: Self = Self(56);
    pub const N: Self = Self(57);
    pub const M: Self = Self(58);
    pub const Comma: Self = Self(59);
    pub const Dot: Self = Self(60);
    pub const Slash: Self = Self(61);
    pub const ShiftRight: Self = Self(62);
    pub const KeypadMultiply: Self = Self(63);
    pub const AltLeft: Self = Self(64);
    pub const Space: Self = Self(65);
    pub const CapsLock: Self = Self(66);
    pub const F1: Self = Self(67);
    pub const F2: Self = Self(68);
    pub const F3: Self = Self(69);
    pub const F4: Self = Self(70);
    pub const F5: Self = Self(71);
    pub const F6: Self = Self(72);
    pub const F7: Self = Self(73);
    pub const F8: Self = Self(74);
    pub const F9: Self = Self(75);
    pub const F10: Self = Self(76);
    pub const NumLock: Self = Self(77);
    pub const ScrollLock: Self = Self(78);
    pub const Keypad7: Self = Self(79);
    pub const Keypad8: Self = Self(80);
    pub const Keypad9: Self = Self(81);
    pub const KeypadMinus: Self = Self(82);
    pub const Keypad4: Self = Self(83);
    pub const Keypad5: Self = Self(84);
    pub const Keypad6: Self = Self(85);
    pub const KeypadPlus: Self = Self(86);
    pub const Keypad1: Self = Self(87);
    pub const Keypad2: Self = Self(88);
    pub const Keypad3: Self = Self(89);
    pub const Keypad0: Self = Self(90);
    pub const KeypadDot: Self = Self(91);
    pub const International: Self = Self(94);
    pub const F11: Self = Self(95);
    pub const F12: Self = Self(96);
    pub const PrintScreen: Self = Self(107);
    pub const AltRight: Self = Self(108);
    pub const Home: Self = Self(110);
    pub const CursorUp: Self = Self(111);
    pub const PageUp: Self = Self(112);
    pub const CursorLeft: Self = Self(113);
    pub const CursorRight: Self = Self(114);
    pub const End: Self = Self(115);
    pub const CursorDown: Self = Self(116);
    pub const PageDown: Self = Self(117);
    pub const Insert: Self = Self(118);
    pub const Delete: Self = Self(119);
    pub const Pause: Self = Self(127);
    pub const LogoLeft: Self = Self(133);
    pub const LogoRight: Self = Self(134);

    pub fn with_super(self) -> KeyBind {
        KeyBind {
            modifiers: KeyModifiers::Super,
            key: self,
        }
    }

    pub fn with_alt(self) -> KeyBind {
        KeyBind {
            modifiers: KeyModifiers::Alt,
            key: self,
        }
    }

    pub fn with_ctrl(self) -> KeyBind {
        KeyBind {
            modifiers: KeyModifiers::Ctrl,
            key: self,
        }
    }

    pub fn with_shift(self) -> KeyBind {
        KeyBind {
            modifiers: KeyModifiers::Shift,
            key: self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyBind {
    pub modifiers: KeyModifiers,
    pub key: XkbKeyCode,
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
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}+{}", modifiers.join("+"), self.key)
        }
    }
}

impl KeyBind {
    pub fn from_pair<M, K>(modifiers: M, key: K) -> Self
    where
        M: Into<KeyModifiers>,
        K: Into<XkbKeyCode>,
    {
        Self {
            modifiers: modifiers.into(),
            key: key.into(),
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

impl std::fmt::Display for XkbKeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Wow! This is really shit
        match self {
            k if *k == Self::Escape => write!(f, "Escape"),
            k if *k == Self::Num1 => write!(f, "Num1"),
            k if *k == Self::Num2 => write!(f, "Num2"),
            k if *k == Self::Num3 => write!(f, "Num3"),
            k if *k == Self::Num4 => write!(f, "Num4"),
            k if *k == Self::Num5 => write!(f, "Num5"),
            k if *k == Self::Num6 => write!(f, "Num6"),
            k if *k == Self::Num7 => write!(f, "Num7"),
            k if *k == Self::Num8 => write!(f, "Num8"),
            k if *k == Self::Num9 => write!(f, "Num9"),
            k if *k == Self::Num0 => write!(f, "Num0"),
            k if *k == Self::Minus => write!(f, "Minus"),
            k if *k == Self::Equal => write!(f, "Equal"),
            k if *k == Self::Backspace => write!(f, "Backspace"),
            k if *k == Self::Tab => write!(f, "Tab"),
            k if *k == Self::Q => write!(f, "Q"),
            k if *k == Self::W => write!(f, "W"),
            k if *k == Self::E => write!(f, "E"),
            k if *k == Self::R => write!(f, "R"),
            k if *k == Self::T => write!(f, "T"),
            k if *k == Self::Y => write!(f, "Y"),
            k if *k == Self::U => write!(f, "U"),
            k if *k == Self::I => write!(f, "I"),
            k if *k == Self::O => write!(f, "O"),
            k if *k == Self::P => write!(f, "P"),
            k if *k == Self::LeftBracket => write!(f, "LeftBracket"),
            k if *k == Self::RightBracket => write!(f, "RightBracket"),
            k if *k == Self::Return => write!(f, "Return"),
            k if *k == Self::CtrlLeft => write!(f, "CtrlLeft"),
            k if *k == Self::A => write!(f, "A"),
            k if *k == Self::S => write!(f, "S"),
            k if *k == Self::D => write!(f, "D"),
            k if *k == Self::F => write!(f, "F"),
            k if *k == Self::G => write!(f, "G"),
            k if *k == Self::H => write!(f, "H"),
            k if *k == Self::J => write!(f, "J"),
            k if *k == Self::K => write!(f, "K"),
            k if *k == Self::L => write!(f, "L"),
            k if *k == Self::Semicolon => write!(f, "Semicolon"),
            k if *k == Self::Apostrophe => write!(f, "Apostrophe"),
            k if *k == Self::Grave => write!(f, "Grave"),
            k if *k == Self::ShiftLeft => write!(f, "ShiftLeft"),
            k if *k == Self::Backslash => write!(f, "Backslash"),
            k if *k == Self::Z => write!(f, "Z"),
            k if *k == Self::X => write!(f, "X"),
            k if *k == Self::C => write!(f, "C"),
            k if *k == Self::V => write!(f, "V"),
            k if *k == Self::B => write!(f, "B"),
            k if *k == Self::N => write!(f, "N"),
            k if *k == Self::M => write!(f, "M"),
            k if *k == Self::Comma => write!(f, "Comma"),
            k if *k == Self::Dot => write!(f, "Dot"),
            k if *k == Self::Slash => write!(f, "Slash"),
            k if *k == Self::ShiftRight => write!(f, "ShiftRight"),
            k if *k == Self::KeypadMultiply => write!(f, "KeypadMultiply"),
            k if *k == Self::AltLeft => write!(f, "AltLeft"),
            k if *k == Self::Space => write!(f, "Space"),
            k if *k == Self::CapsLock => write!(f, "CapsLock"),
            k if *k == Self::F1 => write!(f, "F1"),
            k if *k == Self::F2 => write!(f, "F2"),
            k if *k == Self::F3 => write!(f, "F3"),
            k if *k == Self::F4 => write!(f, "F4"),
            k if *k == Self::F5 => write!(f, "F5"),
            k if *k == Self::F6 => write!(f, "F6"),
            k if *k == Self::F7 => write!(f, "F7"),
            k if *k == Self::F8 => write!(f, "F8"),
            k if *k == Self::F9 => write!(f, "F9"),
            k if *k == Self::F10 => write!(f, "F10"),
            k if *k == Self::NumLock => write!(f, "NumLock"),
            k if *k == Self::ScrollLock => write!(f, "ScrollLock"),
            k if *k == Self::Keypad7 => write!(f, "Keypad7"),
            k if *k == Self::Keypad8 => write!(f, "Keypad8"),
            k if *k == Self::Keypad9 => write!(f, "Keypad9"),
            k if *k == Self::KeypadMinus => write!(f, "KeypadMinus"),
            k if *k == Self::Keypad4 => write!(f, "Keypad4"),
            k if *k == Self::Keypad5 => write!(f, "Keypad5"),
            k if *k == Self::Keypad6 => write!(f, "Keypad6"),
            k if *k == Self::KeypadPlus => write!(f, "KeypadPlus"),
            k if *k == Self::Keypad1 => write!(f, "Keypad1"),
            k if *k == Self::Keypad2 => write!(f, "Keypad2"),
            k if *k == Self::Keypad3 => write!(f, "Keypad3"),
            k if *k == Self::Keypad0 => write!(f, "Keypad0"),
            k if *k == Self::KeypadDot => write!(f, "KeypadDot"),
            k if *k == Self::International => write!(f, "International"),
            k if *k == Self::F11 => write!(f, "F11"),
            k if *k == Self::F12 => write!(f, "F12"),
            k if *k == Self::PrintScreen => write!(f, "PrintScreen"),
            k if *k == Self::AltRight => write!(f, "AltRight"),
            k if *k == Self::Home => write!(f, "Home"),
            k if *k == Self::CursorUp => write!(f, "CursorUp"),
            k if *k == Self::PageUp => write!(f, "PageUp"),
            k if *k == Self::CursorLeft => write!(f, "CursorLeft"),
            k if *k == Self::CursorRight => write!(f, "CursorRight"),
            k if *k == Self::End => write!(f, "End"),
            k if *k == Self::CursorDown => write!(f, "CursorDown"),
            k if *k == Self::PageDown => write!(f, "PageDown"),
            k if *k == Self::Insert => write!(f, "Insert"),
            k if *k == Self::Delete => write!(f, "Delete"),
            k if *k == Self::Pause => write!(f, "Pause"),
            k if *k == Self::LogoLeft => write!(f, "LogoLeft"),
            k if *k == Self::LogoRight => write!(f, "LogoRight"),
            _ => write!(f, "<unknown key {}>", self.0),
        }
    }
}
