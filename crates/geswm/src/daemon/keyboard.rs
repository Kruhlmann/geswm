use smithay::{
    backend::input::{KeyState, Keycode},
    input::keyboard::{FilterResult, KeyboardHandle},
    utils::SERIAL_COUNTER,
};

use crate::{
    cmd::UserCommand,
    daemon::Daemon,
    input::KeyBind,
    server::ServerState,
};

pub type NoKeyboard = ();

pub trait KeyboardHandler {
    fn on_keyboard_event(
        &mut self,
        _time: u64,
        _key: &Keycode,
        _state: &KeyState,
    ) -> Option<UserCommand> {
        None
    }
}

impl<Mouse, Backend, L> KeyboardHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn on_keyboard_event(
        &mut self,
        time: u64,
        key: &Keycode,
        state: &KeyState,
    ) -> Option<UserCommand> {
        let mut cmd_to_return: Option<UserCommand> = None;
        self.keyboard.input(
            &mut self.server_state,
            *key,
            *state,
            SERIAL_COUNTER.next_serial(),
            (time / 1000) as u32,
            |_, modifiers, keysym| match state {
                KeyState::Pressed => {
                    let keybind = KeyBind::from_pair(modifiers, keysym);
                    match self.keybinds.get(&keybind) {
                        Some(command) => {
                            cmd_to_return = Some(command.clone());
                            FilterResult::Intercept(())
                        }
                        None => FilterResult::Forward,
                    }
                }
                KeyState::Released => FilterResult::Forward,
            },
        );
        cmd_to_return
    }
}

impl<Mouse, Backend, L> KeyboardHandler for Daemon<NoKeyboard, Mouse, Backend, L> {}
