// Virtual key constants from HIToolbox/Events.h. The objc2 bindings do not
// currently expose these Carbon constants; keep their SDK names here.
#![allow(non_upper_case_globals)]

pub(super) const kVK_Return: u16 = 0x24;
pub(super) const kVK_Tab: u16 = 0x30;
pub(super) const kVK_Delete: u16 = 0x33;
pub(super) const kVK_Escape: u16 = 0x35;
pub(super) const kVK_RightCommand: u16 = 0x36;
pub(super) const kVK_Command: u16 = 0x37;
pub(super) const kVK_Shift: u16 = 0x38;
pub(super) const kVK_CapsLock: u16 = 0x39;
pub(super) const kVK_Option: u16 = 0x3a;
pub(super) const kVK_Control: u16 = 0x3b;
pub(super) const kVK_RightShift: u16 = 0x3c;
pub(super) const kVK_RightOption: u16 = 0x3d;
pub(super) const kVK_RightControl: u16 = 0x3e;
pub(super) const kVK_ANSI_KeypadClear: u16 = 0x47;
pub(super) const kVK_ANSI_KeypadEnter: u16 = 0x4c;
pub(super) const kVK_Help: u16 = 0x72;
pub(super) const kVK_Home: u16 = 0x73;
pub(super) const kVK_PageUp: u16 = 0x74;
pub(super) const kVK_ForwardDelete: u16 = 0x75;
pub(super) const kVK_End: u16 = 0x77;
pub(super) const kVK_PageDown: u16 = 0x79;
pub(super) const kVK_LeftArrow: u16 = 0x7b;
pub(super) const kVK_RightArrow: u16 = 0x7c;
pub(super) const kVK_DownArrow: u16 = 0x7d;
pub(super) const kVK_UpArrow: u16 = 0x7e;

pub(super) const kVK_F1: u16 = 0x7a;
pub(super) const kVK_F2: u16 = 0x78;
pub(super) const kVK_F3: u16 = 0x63;
pub(super) const kVK_F4: u16 = 0x76;
pub(super) const kVK_F5: u16 = 0x60;
pub(super) const kVK_F6: u16 = 0x61;
pub(super) const kVK_F7: u16 = 0x62;
pub(super) const kVK_F8: u16 = 0x64;
pub(super) const kVK_F9: u16 = 0x65;
pub(super) const kVK_F10: u16 = 0x6d;
pub(super) const kVK_F11: u16 = 0x67;
pub(super) const kVK_F12: u16 = 0x6f;
pub(super) const kVK_F13: u16 = 0x69;
pub(super) const kVK_F14: u16 = 0x6b;
pub(super) const kVK_F15: u16 = 0x71;
pub(super) const kVK_F16: u16 = 0x6a;
pub(super) const kVK_F17: u16 = 0x40;
pub(super) const kVK_F18: u16 = 0x4f;
pub(super) const kVK_F19: u16 = 0x50;
pub(super) const kVK_F20: u16 = 0x5a;

// PC context-menu key; HIToolbox/Events.h does not give this code a name.
pub(super) const CONTEXT_MENU: u16 = 0x6e;
