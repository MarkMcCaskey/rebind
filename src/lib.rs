#[deny(missing_docs)]

extern crate input;
extern crate piston_window;
extern crate rustc_serialize;
extern crate viewport;

mod builder;

use input::{Input, Button, Motion};
use piston_window::Size;
use std::collections::HashMap;
use std::default::Default;
use viewport::Viewport;

pub use builder::InputMapBuilder;

/// Represents a logical action to be bound to a particular button press, e.g.
/// jump, attack, or move forward
pub trait Action: Copy + PartialEq + Eq { }

/// A translated action.
#[derive(Debug, Copy, Clone)]
pub enum Translated<A: Action> {
    /// A keypress event which was bound to an action
    Press(A),

    /// A key release event which was bound to an action
    Release(A),

    /// A translated mouse motion. The logical origin of a translated MouseCursor event
    /// is in the top left corner of the window, and the logical scroll is non-natural.
    /// Relative events are unchanged for now.
    Move(Motion)
}

/// An object which translates piston::input::Input events into input_map::Translated<A> events
#[derive(Clone)]
pub struct InputMap<A: Action> {
    keymap: KeyMap<A>,
    mouse_translator: MouseTranslator
}

impl<A: Action> InputMap<A> {

    /// Creates an empty InputMap.
    pub fn new(size: Size) -> Self {
        InputMap {
            keymap: KeyMap::new(),
            mouse_translator: MouseTranslator::new(size)
        }
    }

    /// Translate an Input into a Translated<A> event
    pub fn translate(&self, input: &Input) -> Option<Translated<A>> {
        macro_rules! translate_button(($but_state:ident, $but_var:ident) => (
            match self.keymap.translate($but_var) {
                Some(act) => Some(Translated::$but_state(act)),
                None => None
            });
        );

        match input {
            &Input::Press(button) => translate_button!(Press, button),
            &Input::Release(button) => translate_button!(Release, button),
            &Input::Move(motion) =>
                Some(Translated::Move(self.mouse_translator.translate(motion))),
            _ => None
        }
    }

    pub fn rebind_button(&mut self, _but: Button, _act: A) {
        // TODO implement
    }

    pub fn add_binding(&mut self, but: Button, act: A) {
        self.keymap.add_mapping(but, act);
    }

    /// Get all the bindings for an action
    pub fn get_bindings_for_action(&self, _act: A) -> ButtonTuple {
        ButtonTuple(None, None, None) // TODO implement
    }

    /// Re-set the mouse bounds size used for calculating mouse events
    pub fn set_size(&mut self, size: Size) {
        self.mouse_translator.viewport_size = size
    }

    /// Re-set the mouse bounds size from a viewport
    pub fn set_size_from_viewport(&mut self, vp: Viewport) {
        self.set_size(Size::from(vp.draw_size));
    }
}

#[derive(Clone)]
struct MouseTranslator {
    pub x_axis_motion_inverted: bool,
    pub y_axis_motion_inverted: bool,
    pub x_axis_scroll_inverted: bool,
    pub y_axis_scroll_inverted: bool,
    pub viewport_size: Size
}

impl MouseTranslator {
    fn new(size: Size) -> Self {
        MouseTranslator {
            x_axis_motion_inverted: false,
            y_axis_motion_inverted: false,
            x_axis_scroll_inverted: false,
            y_axis_scroll_inverted: false,
            viewport_size: size
        }
    }

    fn translate(&self, motion: Motion) -> Motion {
        match motion {
            Motion::MouseCursor(x, y) => {
                let (sw, sh) = {
                    let Size {width, height} = self.viewport_size;
                    (width as f64, height as f64)
                };

                let cx = if self.x_axis_motion_inverted { sw - x } else { x };
                let cy = if self.y_axis_motion_inverted { sh - y } else { y };

                Motion::MouseCursor(cx, cy)
            },
            Motion::MouseScroll(x, y) => {
                let mx = if self.x_axis_scroll_inverted { -1.0f64 } else { 1.0 };
                let my = if self.y_axis_scroll_inverted { -1.0f64 } else { 1.0 };
                Motion::MouseScroll(x * mx, y * my)
            },
            relative => relative
        }
    }
}

#[derive(Clone, Debug)]
struct KeyMap<A: Action> {
    btn_map: HashMap<ButtonTuple, A>
}

impl<A: Action> KeyMap<A> {
    fn new() -> Self {
        KeyMap {
            btn_map: HashMap::new()
        }
    }

    fn add_mapping(&mut self, button: Button, action: A) {
        let mut bt = self.get_bindings_for_action(action).unwrap_or(ButtonTuple::new());
        let bt = if bt.insert_inplace(button) {bt} else {ButtonTuple::new()};
        self.btn_map.insert(bt, action);
    }

    fn get_bindings_for_action(&self, action: A) -> Option<ButtonTuple> {
        self.btn_map.iter().find(|&(_, &a)| a == action).map(|(&bt, _)| bt)
    }

    #[allow(dead_code)]
    fn get_ref_bindings_for_action(&self, action: A) -> Option<&ButtonTuple> {
        self.btn_map.iter().find(|&(_, &a)| a == action).map(|(bt, _)| bt)
    }

    #[cfg(unimplemented)] // This gives a compile error
    fn get_ref_bindings_for_action_mut(&mut self, action: A) -> Option<&mut ButtonTuple> {
        self.btn_map.iter_mut().find(|&(_, &mut a)| a == action).as_mut().map(|&mut (bt, _)| bt)
    }

    fn translate(&self, button: Button) -> Option<A> {
        self.btn_map.iter().find(|&(&bt, _)| bt.contains(button)).map(|(_, &a)| a)
    }
}

/// A three-element tuple of Option<Button>. Used as the key of an InputMap
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ButtonTuple(pub Option<Button>, pub Option<Button>, pub Option<Button>);

impl ButtonTuple {
    fn new() -> Self { Default::default() }

    fn contains(&self, btn: Button) -> bool {
        let sbtn = Some(btn);
        self.0 == sbtn || self.1 == sbtn || self.2 == sbtn
    }

    #[allow(dead_code)]
    fn remove_inplace(&mut self, btn: Button) {
        let sbtn = Some(btn);
        if self.0 == sbtn {self.0 = None}
        if self.1 == sbtn {self.1 = None}
        if self.2 == sbtn {self.2 = None}
    }

    #[allow(dead_code)]
    fn replace_inplace(&mut self, btn_idx: u32, btn: Button) -> bool {
        match btn_idx {
            0 => {self.0 = Some(btn); true},
            1 => {self.1 = Some(btn); true},
            2 => {self.2 = Some(btn); true},
            _ => false
        }
    }

    fn insert_inplace(&mut self, btn: Button) -> bool {
        match self {
            &mut ButtonTuple(a, b, c) if a.is_none() => {*self = ButtonTuple(Some(btn), b, c); true},
            &mut ButtonTuple(a, b, c) if b.is_none() => {*self = ButtonTuple(a, Some(btn), c); true},
            &mut ButtonTuple(a, b, c) if c.is_none() => {*self = ButtonTuple(a, b, Some(btn)); true}
            _ => false
        }
    }
}

impl Default for ButtonTuple {
    fn default() -> Self { ButtonTuple(None, None, None) }
}
