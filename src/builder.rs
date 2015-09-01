use {Action, InputTranslator, InputRebind, MouseTranslationData};
use input::Button;
use window::Size;
use std::convert::Into;
use std::default::Default;

/// Convenience object for constructing an InputMap.
pub struct RebindBuilder<A: Action> {
    input_remappings: Vec<(Button, A)>,
    mouse_data: MouseTranslationData
}

impl<A: Action> RebindBuilder<A> {
    /// Creates a new `RebindBuilder` with the specified viewport size.
    pub fn new(size: Size) -> Self {
        RebindBuilder {
            input_remappings: vec![],
            mouse_data: MouseTranslationData::new(size)
        }
    }

    /// Set whether the x scroll is inverted on the builder (and thus
    /// on the built object).
    pub fn x_scroll_inverted(mut self, invert: bool) -> Self {
        self.mouse_data.x_axis_scroll_inverted = invert;
        self
    }

    /// Returns true if the x scroll is inverted on the builder (and thus
    /// on the built object).
    pub fn get_x_scroll_inverted(&self) -> &bool {
        &self.mouse_data.x_axis_scroll_inverted
    }

    /// Set whether the y scroll is inverted on the builder (and thus
    /// on the built object).
    pub fn y_scroll_inverted(mut self, invert: bool) -> Self {
        self.mouse_data.y_axis_scroll_inverted = invert;
        self
    }

    /// Returns true if the y scroll is inverted on the builder (and thus
    /// on the built object).
    pub fn get_y_scroll_inverted(&self) -> &bool {
        &self.mouse_data.y_axis_scroll_inverted
    }

    /// Set whether the x axis motion is inverted on the builder (and thus
    /// on the built object).
    pub fn x_motion_inverted(mut self, invert: bool) -> Self {
        self.mouse_data.x_axis_motion_inverted = invert;
        self
    }

    /// Returns true if the x axis motion is inverted on the builder (and thus
    /// on the built object).
    pub fn get_x_motion_inverted(&self) -> &bool {
        &self.mouse_data.x_axis_motion_inverted
    }

    /// Set whether the y axis motion is inverted on the builder (and thus
    /// on the built object).
    pub fn y_motion_inverted(mut self, invert: bool) -> Self {
        self.mouse_data.y_axis_motion_inverted = invert;
        self
    }

    /// Returns true if the y axis motion is inverted on the builder (and thus
    /// on the built object).
    pub fn get_y_motion_inverted(&self) -> &bool {
        &self.mouse_data.y_axis_motion_inverted
    }

    /// Sets the viewport size used for mouse position calculations.
    pub fn viewport_size(mut self, size: Size) -> Self {
        self.mouse_data.viewport_size = size;
        self
    }

    /// Returns the currently set viewport size.
    pub fn get_viewport_size(&self) -> &Size {
        &self.mouse_data.viewport_size
    }

    /// Add an association between the Button and Action in the built object.
    pub fn with_action_mapping(mut self, button: Button, action: A) -> Self {
        self.input_remappings.push((button, action));
        self
    }

    /// Creates an `InputTranslator` from this builder object. Consumes self.
    pub fn build_translator(self) -> InputTranslator<A> { self.into() }

    /// Creates an `InputRebind` from this builder object. Consumes self.
    pub fn build_rebind(self) -> InputRebind<A> { self.into() }
}

impl<A: Action> Default for RebindBuilder<A> {
    /// Creates a new `RebindBuilder`. The viewport size is set to (800, 600).
    fn default() -> Self {
        Self::new((800, 600).into())
    }
}

impl<A: Action> Into<InputTranslator<A>> for RebindBuilder<A> {
    fn into(self) -> InputTranslator<A> {
        let mut input_map = InputTranslator::new(self.mouse_data.viewport_size);

        input_map.mouse_translator.data = self.mouse_data;
        input_map.keymap = self.input_remappings.iter().cloned().collect();

        input_map
    }
}

impl<A: Action> Into<InputRebind<A>> for RebindBuilder<A> {
    #[allow(dead_code, unused_variables, unreachable_code)]
    fn into(self) -> InputRebind<A> {
        unimplemented!();

        let mut input_rebind = InputRebind::new(self.mouse_data.viewport_size);

        input_rebind.mouse_data = self.mouse_data;
        //input_rebind.keymap.btn_map = self.input_remappings.iter().map(|x| x.clone()).collect();

        input_rebind
    }
}

