#![warn(missing_docs)]
#![feature(slice_patterns)]

//! rebind
//! =========
//!
//! A library for binding input keys to actions, and modifying mouse behaviour. Keys can be
//! bound to actions, and then translated during runtime. `Keys` are mapped to `Actions` using
//! a `HashMap`, so lookup time is constant.

extern crate input;
extern crate itertools;
extern crate window;
extern crate rustc_serialize;
extern crate viewport;

mod builder;

#[cfg(test)]
mod test;

use input::{Input, Button, Motion};
use window::Size;
use std::collections::HashMap;
use std::default::Default;
use std::cmp::{PartialEq, Eq};
use std::fmt::{Debug, Formatter, Result};
use std::hash::Hash;
use viewport::Viewport;

pub use builder::RebindBuilder;

/// Represents a logical action to be bound to a particular button press, e.g.
/// jump, attack, or move forward. Needs to be hashable, as it is used as a
/// lookup key when rebinding an action to a different button.
pub trait Action: Copy + PartialEq + Eq + Hash { }

/// A translated action.
#[derive(Debug, Copy, Clone, PartialEq)]
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

/// A three-element tuple of Option<Button>. For simplicity, a maximum number of 3
/// buttons can be bound to each action, and this is exposed through the `InputRebind`
/// struct.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ButtonTuple(pub Option<Button>, pub Option<Button>, pub Option<Button>);

impl ButtonTuple {
    /// Creates a new tuple with no buttons in it (equivalent to `Default::default()`).
    pub fn new() -> Self { Default::default() }

    /// Check if the button is in the tuple.
    pub fn contains(&self, button: Button) -> bool {
        let sbtn = Some(button);
        self.0 == sbtn || self.1 == sbtn || self.2 == sbtn
    }

    /// Insert a button into the tuple if there is room, searching from left to right.
    /// If the button is inserted, returns true. Otherwise, if the button is not inserted,
    /// this function returns false.
    pub fn insert_inplace(&mut self, button: Button) -> bool {
        let sbtn = Some(button);
        match self {
            &mut ButtonTuple(a, b, c) if a.is_none() => {*self = ButtonTuple(sbtn, b, c); true},
            &mut ButtonTuple(a, b, c) if b.is_none() => {*self = ButtonTuple(a, sbtn, c); true},
            &mut ButtonTuple(a, b, c) if c.is_none() => {*self = ButtonTuple(a, b, sbtn); true}
            _ => false
        }
    }
}

impl Default for ButtonTuple {
    /// Creates a new tuple with no buttons in it.
    fn default() -> Self { ButtonTuple(None, None, None) }
}

/// An object which translates piston::input::Input events into input_map::Translated<A> events
#[derive(Clone, Debug, PartialEq)]
pub struct InputTranslator<A: Action> {
    keymap: HashMap<Button, A>,
    mouse_translator: MouseTranslator
}

impl<A: Action> InputTranslator<A> {
    /// Creates an empty InputTranslator.
    pub fn new(size: Size) -> Self {
        InputTranslator {
            keymap: HashMap::new(),
            mouse_translator: MouseTranslator::new(size)
        }
    }

    /// Translate an Input into a Translated<A> event. Returns `None` if there is no
    /// action associated with the `Input` variant.
    pub fn translate(&self, input: &Input) -> Option<Translated<A>> {
        macro_rules! translate_button(($but_state:ident, $but_var:ident) => (
            match self.keymap.get(&$but_var).map(|x| *x) {
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

    /// Re-set the mouse bounds size used for calculating mouse events
    pub fn set_size(&mut self, size: Size) {
        self.mouse_translator.data.viewport_size = size
    }

    /// Re-set the mouse bounds size from a viewport
    pub fn set_size_from_viewport(&mut self, vp: Viewport) {
        self.set_size(Size::from(vp.draw_size));
    }
}

#[derive(Clone)]
struct MouseTranslationData {
    x_axis_motion_inverted: bool,
    y_axis_motion_inverted: bool,
    x_axis_scroll_inverted: bool,
    y_axis_scroll_inverted: bool,
    viewport_size: Size
}

impl MouseTranslationData {
    fn new(size: Size) -> Self {
        MouseTranslationData {
            x_axis_motion_inverted: false,
            y_axis_motion_inverted: false,
            x_axis_scroll_inverted: false,
            y_axis_scroll_inverted: false,
            viewport_size: size
        }
    }
}

impl Debug for MouseTranslationData {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}, {}, {}, {}, ({}, {})",
               self.x_axis_motion_inverted,
               self.y_axis_motion_inverted,
               self.x_axis_scroll_inverted,
               self.y_axis_scroll_inverted,
               self.viewport_size.width,
               self.viewport_size.height)
    }
}

impl PartialEq for MouseTranslationData {
    fn eq(&self, other: &Self) -> bool {
        self.x_axis_motion_inverted == other.x_axis_motion_inverted &&
        self.y_axis_motion_inverted == other.y_axis_motion_inverted &&
        self.x_axis_scroll_inverted == other.x_axis_scroll_inverted &&
        self.y_axis_scroll_inverted == other.y_axis_scroll_inverted &&
        self.viewport_size.width    == other.viewport_size.width &&
        self.viewport_size.height   == other.viewport_size.height
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MouseTranslator {
    data: MouseTranslationData
}

impl MouseTranslator {
    fn new(size: Size) -> Self {
        MouseTranslator {
            data: MouseTranslationData::new(size)
        }
    }

    fn translate(&self, motion: Motion) -> Motion {
        match motion {
            Motion::MouseCursor(x, y) => {
                let (sw, sh) = {
                    let Size {width, height} = self.data.viewport_size;
                    (width as f64, height as f64)
                };

                let cx = if self.data.x_axis_motion_inverted { sw - x } else { x };
                let cy = if self.data.y_axis_motion_inverted { sh - y } else { y };

                Motion::MouseCursor(cx, cy)
            },
            Motion::MouseScroll(x, y) => {
                let mx = if self.data.x_axis_scroll_inverted { -1.0f64 } else { 1.0 };
                let my = if self.data.y_axis_scroll_inverted { -1.0f64 } else { 1.0 };
                Motion::MouseScroll(x * mx, y * my)
            },
            relative => relative
        }
    }
}

/// An interface for rebinding keys to actions. This is freely convertable to and
/// from an InputTranslator.
#[derive(Clone, Debug, PartialEq)]
pub struct InputRebind<A: Action> {
    keymap: HashMap<A, ButtonTuple>,
    mouse_data: MouseTranslationData
}

impl<A: Action> InputRebind<A> {
    /// Creates a new InputRebind with no stored Action/ButtonTuple pairs.
    pub fn new(size: Size) -> Self {
        InputRebind {
            keymap: HashMap::new(),
            mouse_data: MouseTranslationData::new(size)
        }
    }

    /// Insert an Action into this InputRebind. If the Action is already in the
    /// InputRebind, then its ButtonTuple will be reset to (None, None, None), and
    /// the old ButtonTuple will be returned.
    pub fn insert_action(&mut self, action: A) -> Option<ButtonTuple> {
        self.keymap.insert(action, ButtonTuple::new())
    }

    /// Insert an Action into this InputRebind, and assign it to the ButtonTuple.
    /// If the Action is already in the InputRebind, the old ButtonTuple will be
    /// returned.
    pub fn insert_action_with_buttons(&mut self, action: A, buttons: ButtonTuple) -> Option<ButtonTuple> {
        self.keymap.insert(action, buttons)
    }

    /// Return a reference to the current ButtonTuple stored for an action. If the action
    /// is not stored in this InputRebind, then `None` will be returned.
    pub fn get_bindings(&self, action: &A) -> Option<&ButtonTuple> {
        self.keymap.get(action)
    }

    /// Returns a mutable reference to the current ButtonTuple stored for an action. If the
    /// action is not stored in this InputRebind, then `None` will be returned.
    pub fn get_bindings_mut(&mut self, action: &mut A) -> Option<&mut ButtonTuple> {
        self.keymap.get_mut(action)
    }

    /// Returns a reference to the boolean which represents whether x axis scrolling is inverted.
    pub fn get_x_scroll_inverted(&self) -> &bool {
        &self.mouse_data.x_axis_scroll_inverted
    }

    /// Returns a mutable reference to the boolean which represents whether x axis scrolling is inverted.
    pub fn get_x_scroll_inverted_mut(&mut self) -> &mut bool {
        &mut self.mouse_data.x_axis_scroll_inverted
    }

    /// Returns a reference to the boolean which represents whether y axis scrolling is inverted.
    pub fn get_y_scroll_inverted(&self) -> &bool {
        &self.mouse_data.y_axis_scroll_inverted
    }

    /// Returns a mutable reference to the boolean which represents whether y axis scrolling is inverted.
    pub fn get_y_scroll_inverted_mut(&mut self) -> &mut bool {
        &mut self.mouse_data.y_axis_scroll_inverted
    }

    /// Returns a reference to the boolean which represents whether mouse movement along the x axis is
    /// inverted.
    pub fn get_x_motion_inverted(&self) -> &bool {
        &self.mouse_data.x_axis_motion_inverted
    }

    /// Returns a mutable reference to the boolean which represents whether mouse movement along the
    /// x axis is inverted.
    pub fn get_x_motion_inverted_mut(&mut self) -> &mut bool {
        &mut self.mouse_data.x_axis_motion_inverted
    }

    /// Returns a reference to the boolean which represents whether mouse movement along the y axis is
    /// inverted.
    pub fn get_y_motion_inverted(&self) -> &bool {
        &self.mouse_data.y_axis_motion_inverted
    }

    /// Returns a mutable reference to the boolean which represents whether mouse movement along the
    /// y axis is inverted.
    pub fn get_y_motion_inverted_mut(&mut self) -> &mut bool {
        &mut self.mouse_data.y_axis_motion_inverted
    }

    /// Returns a reference to the currently stored viewport size used for calculating the imaginary mouse
    /// position.
    pub fn get_viewport_size(&self) -> &Size {
        &self.mouse_data.viewport_size
    }

    /// Returns a mutable reference to the currently stored viewport size used for calculating the imaginary
    /// mouse position.
    pub fn get_viewport_size_mut(&mut self) -> &mut Size {
        &mut self.mouse_data.viewport_size
    }
}

impl<A: Action> Default for InputRebind<A> {
    /// Creates an `InputRebind` with no pairs. In addition, the viewport size is set to [800, 600].
    fn default() -> Self {
        InputRebind::new((800, 600).into())
    }
}

impl<A: Action> Into<InputTranslator<A>> for InputRebind<A> {
    fn into(self) -> InputTranslator<A> {
        let mut input_translator = InputTranslator::new(self.mouse_data.viewport_size);
        input_translator.mouse_translator.data = self.mouse_data;
        input_translator.keymap = self.keymap.values()
                                             .flat_map(|&bt| bt.into_iter())
                                             .filter_map(|x| x)
                                             .zip(self.keymap.keys().cloned())
                                             .collect();

        input_translator
    }
}

impl<A: Action> Into<InputRebind<A>> for InputTranslator<A> {
    fn into(self) -> InputRebind<A> {
        use itertools::Itertools;

        let mut input_rebind = InputRebind::new(self.mouse_translator.data.viewport_size);
        input_rebind.mouse_data = self.mouse_translator.data;
        input_rebind.keymap = self.keymap.keys()
                                         .map(|x| vec![Some(x)])
                                         .coalesce(|b0, b1| if b0 == b1 {
                                                 Ok(b0.into_iter().chain(b1).collect())
                                             } else {
                                                 Err((b0, b1))
                                             })
                                         .map(|s| if let [b0, b1, b2] = &s.iter()
                                                                          .fuse()
                                                                          .take(3)
                                                                          .map(|x| x.cloned())
                                                                          .collect::<Vec<_>>()[..] {
                                                 ButtonTuple(b0, b1, b2)
                                             } else {
                                                 unreachable!();
                                             })
                                         .zip(self.keymap.values().cloned())
                                         .map(|(k, v)| (v, k))
                                         .collect();

        input_rebind
    }
}

impl ButtonTuple {
    fn into_iter(self) -> ButtonTupleIterator {
        ButtonTupleIterator {
            button_tuple: self,
            i: 0
        }
    }
}

struct ButtonTupleIterator {
    button_tuple: ButtonTuple,
    i: usize
}

impl Iterator for ButtonTupleIterator {
    type Item = Option<Button>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        self.i += 1;
        match i {
            0 => Some(self.button_tuple.0),
            1 => Some(self.button_tuple.1),
            2 => Some(self.button_tuple.2),
            _ => None
        }
    }
}
