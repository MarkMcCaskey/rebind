rebind
======

A library for binding input keys to actions, and modifying mouse behaviour. Keys can be bound to
actions, and then translated during runtime. `Keys` are mapped to `Actions` using a `HashMap`, so
lookup time is constant.

Example
-------

```rust
extern crate glutin_window;
extern crate piston;
extern crate rebind;

use glutin_window::GlutinWindow;
use piston::event_loop::Events;
use piston::input::Event;
use piston::input::Button::Keyboard;
use piston::input::keyboard::Key;
use piston::window::WindowSettings;
use rebind::{Action, RebindBuilder, Translated};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum MyAction {
    Action1, Action2
}

impl Action for MyAction { }

fn main() {
    let window: GlutinWindow = WindowSettings::new("rebind-example", (800, 600))
        .build()
        .unwrap_or_else(|e| panic!("Could not create window: {}", e));

    let translator = RebindBuilder::<MyAction>::new((800, 600).into())
        .with_action_mapping(Keyboard(Key::D1), MyAction::Action1)
        .with_action_mapping(Keyboard(Key::A),  MyAction::Action1)
        .with_action_mapping(Keyboard(Key::D2), MyAction::Action2)
        .with_action_mapping(Keyboard(Key::B),  MyAction::Action2)
        .build_translator();

    for e in window.events() {
        if let Event::Input(ref i) = e {
            if let Some(a) = translator.translate(i) {
                match a {
                    Translated::Press(MyAction::Action1) => {
                        println!("Action 1 pressed!");
                    },
                    Translated::Press(MyAction::Action2) => {
                        println!("Action 2 pressed!");
                    },
                    _ => { }
                }
            }
        }
    }
}
```

rebind_plugins
--------------

This library has a companion crate called `rebind_plugins`. If you are using the nighly compiler, then
you can use this crate, which contains a special `derive` annotation for declaring `Action`s:

```rust
#![feature(plugin)]
#![plugin(rebind_macros)]

#[derive(Action)]
enum MyAction {
    Action1, Action2
}

// ... rest of example as normal
```

Main improvements to be made:
-----------------------------

* Implement conversion from InputTranslator to InputRebind
* Improve the API: Is the distinction between InputRebind and InputTranslator necessary or useful?
* Add serialisation
* Add mouse sensitivity options
* Add more tests/benchmarks
* Improve documentation
* Change the internal lookup from a `HashMap` to an array, where actions are stored in an array at positions
  corresponding to the numeric value of the key. This should be faster than using a `HashMap` and collisions/
  algorithmic DOS attacks are not a concern, although the size of the map is usually very small and lookups
  are relatively quick, so this is probably not a problem in practice.

Contributions welcome.
