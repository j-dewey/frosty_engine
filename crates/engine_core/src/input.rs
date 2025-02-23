use std::{any::TypeId, sync::OnceLock, time::Instant};

use action::{
    Action1, Action2, Action3, BackwardAction, ForwardAction, InputAction, LeftAction, RightAction,
};
use hashbrown::{HashMap, HashSet};
use render::winit::{
    self,
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
    // how mcuh time the last frame lasted, in seconds
    dt: f64,
    // at what point in time the last frame was
    last_frame: Instant,
    // window size (for screen spacew coordinates)
    win_size: winit::dpi::PhysicalSize<f64>,
}

// Set up the input handler static variable. This MUST be called
// before any other input methods are. Subsequent calls to this
// method will return HandlerAlreadyInit error and not change
// the current handler.
#[allow(static_mut_refs)]
pub unsafe fn init_input(win_size: winit::dpi::PhysicalSize<u32>) -> Result<(), InputError> {
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
        dt: 0.0,
        last_frame: Instant::now(),
        win_size: winit::dpi::PhysicalSize {
            width: win_size.width as i32 as f64,
            height: win_size.height as i32 as f64,
        },
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

// Get how long the last frame took
#[allow(static_mut_refs)]
pub fn get_dt_seconds() -> Result<f64, InputError> {
    unsafe {
        match INPUT_HANDLER.get() {
            Some(ih) => Ok(ih.dt),
            None => Err(InputError::HandlerUninit),
        }
    }
}

// Get whether a key is pressed or not
#[allow(static_mut_refs)]
pub fn get_key(key: &KeyCode) -> Result<bool, InputError> {
    unsafe {
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
}

// Get the current mouse position in pixel coordinates
#[allow(static_mut_refs)]
pub fn get_mouse_pos() -> Result<PhysicalPosition<f64>, InputError> {
    unsafe {
        match INPUT_HANDLER.get() {
            Some(ih) => Ok(ih.mouse_position),
            None => Err(InputError::HandlerUninit),
        }
    }
}

// Get the current mouse position in screen space coordinate [-1.0, 1.0]
#[allow(static_mut_refs)]
pub fn get_mouse_pos_screen_space() -> Result<PhysicalPosition<f64>, InputError> {
    unsafe {
        match INPUT_HANDLER.get() {
            Some(ih) => {
                let mpos = ih.mouse_position;
                let screen = ih.win_size;
                Ok(PhysicalPosition {
                    x: (mpos.x / screen.width) * 2.0 - 1.0,
                    y: (mpos.y / screen.height) * -2.0 + 1.0,
                })
            }
            None => Err(InputError::HandlerUninit),
        }
    }
}

// Returns whether a mouse button is down or not.
// If looking for a new press, use get_new_mouse_press instead
#[allow(static_mut_refs)]
pub fn get_mouse_press(button: MouseButton) -> Result<bool, InputError> {
    unsafe {
        match INPUT_HANDLER.get().unwrap().mouse_states.get(&button) {
            Some(state) => Ok(*state),
            None => Err(InputError::UnrecognizedMouseButton),
        }
    }
}

// Returns whether a mouse button was *just* pressed
// If looking for a held button, use get_mouse_press instead
#[allow(static_mut_refs)]
pub fn get_new_mouse_press(button: MouseButton) -> Result<bool, InputError> {
    unsafe {
        match INPUT_HANDLER.get() {
            Some(ih) => Ok(ih
                .frame_events
                .get(&InputEvent::MousePress(button))
                .is_some()),
            None => Err(InputError::HandlerUninit),
        }
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

// clear frame update buffer and recalulate dt
#[allow(static_mut_refs)]
pub unsafe fn flush_frame_updates() -> Result<(), InputError> {
    match INPUT_HANDLER.get_mut() {
        Some(ih) => {
            ih.frame_events.clear();
            let now = Instant::now();
            ih.dt = (now - ih.last_frame).as_secs_f64();
            ih.last_frame = now;
            Ok(())
        }
        None => Err(InputError::HandlerUninit),
    }
}

#[allow(static_mut_refs)]
pub unsafe fn resize(new_win_size: winit::dpi::PhysicalSize<u32>) -> Result<(), InputError> {
    let ih = INPUT_HANDLER.get_mut().ok_or(InputError::HandlerUninit)?;
    ih.win_size = winit::dpi::PhysicalSize {
        width: new_win_size.width as i32 as f64,
        height: new_win_size.height as i32 as f64,
    };
    Ok(())
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
