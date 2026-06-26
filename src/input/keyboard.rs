use smithay::input::keyboard::ModifiersState;

pub struct Key;

#[allow(non_upper_case_globals)]
impl Key {
    pub const Shift: u32 = 0b10000000000000000000000000000000;
    pub const Ctrl: u32 = 0b01000000000000000000000000000000;
    pub const Alt: u32 = 0b00100000000000000000000000000000;
    pub const Super: u32 = 0b00010000000000000000000000000000;

    pub const Empty: u32 = 0;
    pub const Escape: u32 = 9;
    pub const Num1: u32 = 10;
    pub const Num2: u32 = 11;
    pub const Num3: u32 = 12;
    pub const Num4: u32 = 13;
    pub const Num5: u32 = 14;
    pub const Num6: u32 = 15;
    pub const Num7: u32 = 16;
    pub const Num8: u32 = 17;
    pub const Num9: u32 = 18;
    pub const Num0: u32 = 19;
    pub const Minus: u32 = 20;
    pub const Equal: u32 = 21;
    pub const Backspace: u32 = 22;
    pub const Tab: u32 = 23;
    pub const Q: u32 = 24;
    pub const W: u32 = 25;
    pub const E: u32 = 26;
    pub const R: u32 = 27;
    pub const T: u32 = 28;
    pub const Y: u32 = 29;
    pub const U: u32 = 30;
    pub const I: u32 = 31;
    pub const O: u32 = 32;
    pub const P: u32 = 33;
    pub const LeftBracket: u32 = 34;
    pub const RightBracket: u32 = 35;
    pub const Return: u32 = 36;
    pub const CtrlLeft: u32 = 37;
    pub const A: u32 = 38;
    pub const S: u32 = 39;
    pub const D: u32 = 40;
    pub const F: u32 = 41;
    pub const G: u32 = 42;
    pub const H: u32 = 43;
    pub const J: u32 = 44;
    pub const K: u32 = 45;
    pub const L: u32 = 46;
    pub const Semicolon: u32 = 47;
    pub const Apostrophe: u32 = 48;
    pub const Grave: u32 = 49;
    pub const ShiftLeft: u32 = 50;
    pub const Backslash: u32 = 51;
    pub const Z: u32 = 52;
    pub const X: u32 = 53;
    pub const C: u32 = 54;
    pub const V: u32 = 55;
    pub const B: u32 = 56;
    pub const N: u32 = 57;
    pub const M: u32 = 58;
    pub const Comma: u32 = 59;
    pub const Dot: u32 = 60;
    pub const Slash: u32 = 61;
    pub const ShiftRight: u32 = 62;
    pub const KeypadMultiply: u32 = 63;
    pub const AltLeft: u32 = 64;
    pub const Space: u32 = 65;
    pub const CapsLock: u32 = 66;
    pub const F1: u32 = 67;
    pub const F2: u32 = 68;
    pub const F3: u32 = 69;
    pub const F4: u32 = 70;
    pub const F5: u32 = 71;
    pub const F6: u32 = 72;
    pub const F7: u32 = 73;
    pub const F8: u32 = 74;
    pub const F9: u32 = 75;
    pub const F10: u32 = 76;
    pub const NumLock: u32 = 77;
    pub const ScrollLock: u32 = 78;
    pub const Keypad7: u32 = 79;
    pub const Keypad8: u32 = 80;
    pub const Keypad9: u32 = 81;
    pub const KeypadMinus: u32 = 82;
    pub const Keypad4: u32 = 83;
    pub const Keypad5: u32 = 84;
    pub const Keypad6: u32 = 85;
    pub const KeypadPlus: u32 = 86;
    pub const Keypad1: u32 = 87;
    pub const Keypad2: u32 = 88;
    pub const Keypad3: u32 = 89;
    pub const Keypad0: u32 = 90;
    pub const KeypadDot: u32 = 91;
    pub const International: u32 = 94;
    pub const F11: u32 = 95;
    pub const F12: u32 = 96;
    pub const PrintScreen: u32 = 107;
    pub const AltRight: u32 = 108;
    pub const Home: u32 = 110;
    pub const CursorUp: u32 = 111;
    pub const PageUp: u32 = 112;
    pub const CursorLeft: u32 = 113;
    pub const CursorRight: u32 = 114;
    pub const End: u32 = 115;
    pub const CursorDown: u32 = 116;
    pub const PageDown: u32 = 117;
    pub const Insert: u32 = 118;
    pub const Delete: u32 = 119;
    pub const Pause: u32 = 127;
    pub const LogoLeft: u32 = 133;
    pub const LogoRight: u32 = 134;

    pub fn with_modifiers(key: u32, modifiers: &ModifiersState) -> u32 {
        let mut key = key;
        if modifiers.shift {
            key |= Key::Shift;
        }
        if modifiers.ctrl {
            key |= Key::Ctrl;
        }
        if modifiers.alt {
            key |= Key::Alt;
        }
        if modifiers.logo {
            key |= Key::Super;
        }
        key
    }

    pub fn display(key: u32) -> String {
        let mut modifiers: Vec<String> = Vec::new();
        if key & Key::Super != 0 {
            modifiers.push("Super".to_string());
        }
        if key & Key::Alt != 0 {
            modifiers.push("Alt".to_string());
        }
        if key & Key::Ctrl != 0 {
            modifiers.push("Ctrl".to_string());
        }
        if key & Key::Shift != 0 {
            modifiers.push("Shift".to_string());
        }
        let modifiers_str = if modifiers.is_empty() {
            "".to_string()
        } else {
            format!("{}+", modifiers.join("+"))
        };
        format!("{}{:#06x}", modifiers_str, key & 0xFFFFFF)
    }
}
