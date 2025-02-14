use std::{any::TypeId, sync::OnceLock};

use action::{
    Action1, Action2, Action3, BackwardAction, ForwardAction, InputAction, LeftAction, RightAction,
};
use hashbrown::{HashMap, HashSet};
use render::winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

mod action;
mod keys;

// SAFETY:
//      The main issue when using a static mutable variables is two fold:
//      1) All threads have access to it, so data races can occur
//      2) Pointer data can be invalidated, causing use after frees
//
static mut INPUT_HANDLER: OnceLock<InputHandler> = OnceLock::new();

#[derive(Debug)]
pub enum InputError {
    HandlerAlreadyInit,
    HandlerUninit,
    UnrecognizedAction,
    UnrecognizedKeyCode,
    UnrecognizedMouseButton,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputEvent {
    KeyPress(KeyCode),
    MousePress(MouseButton),
}

#[derive(Debug)]
pub struct InputHandler {
    // This states which keys are held down
    key_states: HashMap<KeyCode, bool>,
    // This maps a specific action to its
    // associated key
    actions: HashMap<TypeId, KeyCode>,
    // A list of events that are new from this frame
    frame_events: HashSet<InputEvent>,
    // The position of the mouse in pixel coordinates
    mouse_position: PhysicalPosition<f64>,
    // Whether mice buttons are currently priced
    mouse_states: HashMap<MouseButton, bool>,
    // How much the
}

// Set up the input handler static variable. This MUST be called
// before any other input methods are. Subsequent calls to this
// method will return HandlerAlreadyInit error and not change
// the current handler.
#[allow(static_mut_refs)]
pub unsafe fn init_input() -> Result<(), InputError> {
    if INPUT_HANDLER.get().is_some() {
        return Err(InputError::HandlerAlreadyInit);
    }
    let mut mouse_states: HashMap<MouseButton, bool> = HashMap::new();
    mouse_states.insert(MouseButton::Left, false);
    mouse_states.insert(MouseButton::Right, false);
    mouse_states.insert(MouseButton::Middle, false);

    let ih = InputHandler {
        key_states: keys::create_keyboard_hash_map(),
        actions: HashMap::new(),
        frame_events: HashSet::new(),
        mouse_position: PhysicalPosition { x: 0.0, y: 0.0 },
        mouse_states,
    };
    INPUT_HANDLER.set(ih).expect("Failed to load input");
    Ok(())
}

// Register an action so that it can be tracked with teh get_action
// function.
#[allow(static_mut_refs)]
pub unsafe fn register_action<A: InputAction>(key: KeyCode) -> Result<(), InputError> {
    match INPUT_HANDLER.get_mut() {
        Some(inp) => {
            inp.actions.insert(TypeId::of::<A>(), key);
            Ok(())
        }
        None => Err(InputError::HandlerUninit),
    }
}

// Registers some basic actions for 2d or 3d games.
// Movement:
//      Forward  - W
//      Backward - S
//      Left     - A
//      Right    - D
// General:
//      Action1  - Q
//      Action2  - E
//      Action3  - C
#[allow(static_mut_refs)]
pub unsafe fn register_general_actions() -> Result<(), InputError> {
    if let Some(inp) = INPUT_HANDLER.get_mut() {
        inp.actions
            .insert(TypeId::of::<ForwardAction>(), KeyCode::KeyW);
        inp.actions
            .insert(TypeId::of::<BackwardAction>(), KeyCode::KeyS);
        inp.actions
            .insert(TypeId::of::<LeftAction>(), KeyCode::KeyA);
        inp.actions
            .insert(TypeId::of::<RightAction>(), KeyCode::KeyD);
        inp.actions.insert(TypeId::of::<Action1>(), KeyCode::KeyQ);
        inp.actions.insert(TypeId::of::<Action2>(), KeyCode::KeyE);
        inp.actions.insert(TypeId::of::<Action3>(), KeyCode::KeyC);
        Ok(())
    } else {
        Err(InputError::HandlerUninit)
    }
}

// Get whether a key is pressed or not
#[allow(static_mut_refs)]
pub unsafe fn get_key(key: &KeyCode) -> Result<bool, InputError> {
    match INPUT_HANDLER
        .get()
        .expect("Attempted getting key before INPUT_HANDLER init")
        .key_states
        .get(key)
    {
        Some(state) => Ok(*state),
        None => Err(InputError::UnrecognizedKeyCode),
    }
}

#[allow(static_mut_refs)]
pub unsafe fn get_mouse_press(button: MouseButton) -> Result<bool, InputError> {
    match INPUT_HANDLER.get().unwrap().mouse_states.get(&button) {
        Some(state) => Ok(*state),
        None => Err(InputError::UnrecognizedMouseButton),
    }
}

// Get the state of a specific action. Useful for rebinding
#[allow(static_mut_refs)]
pub fn get_action<A: InputAction>() -> Result<bool, InputError> {
    unsafe {
        match INPUT_HANDLER.get() {
            Some(ih) => {
                let key = match ih.actions.get(&TypeId::of::<A>()) {
                    Some(key) => key,
                    None => return Err(InputError::UnrecognizedAction),
                };
                get_key(key)
            }
            None => Err(InputError::HandlerUninit),
        }
    }
}

// Set both a keys state and add it to the frame events list if possible
pub unsafe fn set_key(ih: &mut InputHandler, key: KeyCode, state: bool) -> Option<()> {
    if !*(ih.key_states.get(&key)?) && state {
        ih.frame_events.insert(InputEvent::KeyPress(key));
    }
    ih.key_states.insert(key, state);
    Some(())
}

//
#[allow(static_mut_refs)]
pub unsafe fn receive_window_input(event: &WindowEvent) -> bool {
    let ih = INPUT_HANDLER
        .get_mut()
        .expect("Failed to init Input Handler before receiving input");
    match event {
        // return from match is return from method
        // if this iterates over each key binding then keyboard input will
        // have O(cn) where c is the number of bindings and n is the number of inputs
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    ..
                },
            ..
        } => {
            set_key(ih, *keycode, *state == ElementState::Pressed);
            true
        }
        WindowEvent::MouseInput { state, button, .. } => {
            // A button is true if it is being pressed
            let pressed = *state == ElementState::Pressed;
            // having this here is icky, but will work for now
            if pressed && !unsafe { *ih.mouse_states.get(button).unwrap_unchecked() } {
                ih.frame_events.insert(InputEvent::MousePress(*button));
            }
            ih.mouse_states.insert(*button, pressed);
            true
        }
        WindowEvent::CursorMoved { position, .. } => {
            // find dx and dy
            let new_x = position.x;
            let new_y = position.y;
            let old_x = ih.mouse_position.x;
            let old_y = ih.mouse_position.y;
            let dx = new_x - old_x;
            let dy = new_y - old_y;
            ih.mouse_position = *position;
            true
        }
        /*
        WindowEvent::TouchpadPressure { stage, .. } => {
            // this should be treated like a left mouse button press
            println!("{:?}", unsafe {
                *ih.mouse_states.get(&MouseButton::Left).unwrap_unchecked()
            });
            if *stage > 0
                && !unsafe { *ih.mouse_states.get(&MouseButton::Left).unwrap_unchecked() }
            {
                ih.frame_update
                    .push(InputEvent::MousePress(MouseButton::Left));
                println!("Click");
            }
            ih.mouse_states.insert(MouseButton::Left, *stage > 0);
            true
        }
        */
        _ => false, // window event is untracked
    }
}
