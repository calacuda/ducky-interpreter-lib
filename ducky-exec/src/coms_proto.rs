use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LedState {
    RED,
    GREEN,
    OFF,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecialKey {
    Enter,
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrntScrn,
    Ins,
    Del,
    BackSpace,
    Tab,
    Home,
    End,
    CapsLock,
    PgUp,
    LeftShift,
    RightShift,
    PgDown,
    LeftCtrl,
    LeftSuper,
    LeftAlt,
    Space,
    RightAlt,
    RightCtrl,
    RightSuper,
    Fn,
    Up,
    Down,
    Left,
    Right,
    PauseBreak,
    Menu,
    NumLock,
    ScrollLock,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Command {
    LED(LedState),
    TypeChar(u8),
    TriggerKey(SpecialKey),
    HoldKey(SpecialKey),
    ReleaseKey(SpecialKey),
}
