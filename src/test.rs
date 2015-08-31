#![allow(dead_code, unused_variables)]

use {Action, ButtonTuple, InputTranslator, RebindBuilder, InputRebind, Translated};
use input::Input;
use input::Button::Keyboard;
use input::keyboard::Key;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
enum TestAction {
    Action1, Action2, Action3, Action4, Action5,
    Action6, Action7, Action8, Action9, Action10
}

impl Action for TestAction { }

type TestBuilder = RebindBuilder<TestAction>;
type TestTranslator = InputTranslator<TestAction>;
type TestRebind = InputRebind<TestAction>;

fn create_prepopulated_builder() -> TestBuilder {
    RebindBuilder::default()
        .with_action_mapping(Keyboard(Key::Up),    TestAction::Action1)
        .with_action_mapping(Keyboard(Key::W),     TestAction::Action1)
        .with_action_mapping(Keyboard(Key::Down),  TestAction::Action2)
        .with_action_mapping(Keyboard(Key::S),     TestAction::Action2)
        .with_action_mapping(Keyboard(Key::Left),  TestAction::Action3)
        .with_action_mapping(Keyboard(Key::A),     TestAction::Action3)
        .with_action_mapping(Keyboard(Key::Right), TestAction::Action4)
        .with_action_mapping(Keyboard(Key::D),     TestAction::Action4)
}

#[test]
fn test_translator_get_action_from_buttonpress() {
    let translator = create_prepopulated_builder().build_translator();

    assert_eq!(translator.translate(&Input::Press(Keyboard(Key::Down))).unwrap(),
               Translated::Press(TestAction::Action2));

    assert_eq!(translator.translate(&Input::Press(Keyboard(Key::D))).unwrap(),
               Translated::Press(TestAction::Action4));

    assert_eq!(translator.translate(&Input::Press(Keyboard(Key::Left))).unwrap(),
               Translated::Press(TestAction::Action3));

    assert_eq!(translator.translate(&Input::Press(Keyboard(Key::W))).unwrap(),
               Translated::Press(TestAction::Action1));
}

#[test]
fn test_conversion_from_rebind_to_translator() {
    let translator = create_prepopulated_builder().build_translator();

    let translator_clone = translator.clone();
    let converted_translator = Into::<TestTranslator>::into(
            Into::<TestRebind>::into(translator));

    assert_eq!(translator_clone, converted_translator);
}

#[test]
fn test_add_button_to_translator_using_rebind() {
    use input::Button;
    const Q_KEY: Button = Keyboard(Key::Q);
    const E_KEY: Button = Keyboard(Key::E);

    let translator = create_prepopulated_builder().build_translator();
    let mut rebind = translator.into_rebind();
    rebind.insert_action_with_buttons(TestAction::Action5, ButtonTuple(Some(Q_KEY), Some(E_KEY), None));

    let translator = rebind.into_translator();

    assert_eq!(translator.translate(&Input::Press(Q_KEY)), Some(Translated::Press(TestAction::Action5)));
    assert_eq!(translator.translate(&Input::Press(E_KEY)), Some(Translated::Press(TestAction::Action5)));
}
