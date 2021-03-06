use std::collections::HashMap;

use penrose::{WindowManager, XcbConnection, __test_helpers::KeyCode};

pub struct KeyMod;

impl KeyMod {
    pub const NONE: u16 = 0;
    pub const ALT: u16 = 8;
    pub const SUPER: u16 = 64;
    pub const SHIFT: u16 = 1;
    pub const CTRL: u16 = 4;
}

pub type BindingFn = dyn FnMut(&mut WindowManager<XcbConnection>) -> penrose::Result<()>;
pub type KnownCodes = HashMap<String, u8>;

pub struct BetterKeyBindings {
    codes: KnownCodes,
    bindings: HashMap<&'static str, Box<BindingFn>>,
}

impl Default for BetterKeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl BetterKeyBindings {
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
        key: &'static str,
        func: impl FnMut(&mut WindowManager<XcbConnection>) -> penrose::Result<()> + 'static,
    ) {
        self.bindings.insert(key, Box::new(func));
    }

    pub fn into_penrose_bindings(self) -> HashMap<KeyCode, Box<BindingFn>> {
        self.bindings
            .into_iter()
            .map(|(key_str, mut func)| {
                let key = Self::key_parse(&self.codes, key_str);

                let penrose_fn: Box<BindingFn> =
                    Box::new(move |wm: &mut WindowManager<XcbConnection>| func(wm));

                (key, penrose_fn)
            })
            .collect::<HashMap<_, _>>()
    }
}
