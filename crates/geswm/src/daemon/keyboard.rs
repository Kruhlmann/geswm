use smithay::{
    backend::input::{KeyState, Keycode},
    input::keyboard::{FilterResult, KeyboardHandle},
    utils::SERIAL_COUNTER,
};

use crate::{
    daemon::{Daemon, bind::KeyBind},
    server::ServerState,
};

pub type NoKeyboard = ();

pub trait KeyboardHandler {
    fn on_keyboard_event(&mut self, _time: u64, _key: &Keycode, _state: &KeyState) {}
}

impl<Mouse, Backend, L> KeyboardHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn on_keyboard_event(&mut self, time: u64, key: &Keycode, state: &KeyState) {
        tracing::info!(?key, "kjey");
        self.keyboard.input(
            &mut self.server_state,
            *key,
            *state,
            SERIAL_COUNTER.next_serial(),
            (time / 1000) as u32,
            |state, modifiers, keysym| match self.keybinds.get(&KeyBind {
                modifiers: modifiers.into(),
                key: keysym.raw_code().raw().into(),
            }) {
                Some(command) => {
                    command.execute(state.socket_name());
                    FilterResult::Intercept(())
                }
                None => {
                    tracing::info!(?modifiers, ?keysym, "No keybind found for this key");
                    FilterResult::Forward
                }
            },
        );
    }
}

impl<Mouse, Backend, L> KeyboardHandler for Daemon<NoKeyboard, Mouse, Backend, L> {}
