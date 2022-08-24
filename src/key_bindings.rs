use std::collections::HashMap;

use penrose::{
    core::{bindings::KeyCode, xconnection::XConn},
    WindowManager,
};

pub struct KeyMod;

impl KeyMod {
    pub const NONE: u16 = 0;
    pub const ALT: u16 = 8;
    pub const SUPER: u16 = 64;
    pub const SHIFT: u16 = 1;
    pub const CTRL: u16 = 4;
}

pub type BindingFn<X> = dyn FnMut(&mut WindowManager<X>) -> penrose::Result<()>;
pub type KnownCodes = HashMap<String, u8>;

pub struct BetterKeyBindings<X: XConn + 'static> {
    codes: KnownCodes,
    bindings: HashMap<String, Box<BindingFn<X>>>,
}

impl<X: XConn + 'static> Default for BetterKeyBindings<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X: XConn + 'static> BetterKeyBindings<X> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            codes: penrose::core::helpers::keycodes_from_xmodmap()
                .into_iter()
                .map(|(string, code)| (string.to_lowercase(), code))
                .collect::<KnownCodes>(),
            bindings: HashMap::new(),
        }
    }

    fn key_parse(codes: &KnownCodes, key_str: &str) -> KeyCode {
        let mut parts = key_str.split(' ').collect::<Vec<_>>();

        let key = *codes
            .get(&parts.remove(parts.len() - 1).to_lowercase())
            .unwrap();

        let mut key_mod = KeyMod::NONE;

        for modifier in parts {
            key_mod |= match modifier {
                "super" => KeyMod::SUPER,
                "alt" => KeyMod::ALT,
                "shift" => KeyMod::SHIFT,
                "ctrl" => KeyMod::CTRL,
                _ => KeyMod::NONE,
            };
        }

        KeyCode {
            mask: key_mod,
            code: key,
        }
    }

    pub fn add(
        &mut self,
        key: impl Into<String>,
        func: impl FnMut(&mut WindowManager<X>) -> penrose::Result<()> + 'static,
    ) {
        self.bindings.insert(key.into(), Box::new(func));
    }

    #[must_use]
    pub fn into_penrose_bindings(self) -> HashMap<KeyCode, Box<BindingFn<X>>> {
        self.bindings
            .into_iter()
            .map(|(key_str, mut func)| {
                let key = Self::key_parse(&self.codes, &key_str);

                let penrose_fn: Box<BindingFn<X>> =
                    Box::new(move |wm: &mut WindowManager<X>| func(wm));

                (key, penrose_fn)
            })
            .collect::<HashMap<_, _>>()
    }
}
