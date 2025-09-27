// Marker trait to denote that an object represents
// some kind of action by the user
pub trait InputAction: 'static {}

// These are generic actions which can mean anything to games
pub struct Action1;
impl InputAction for Action1 {}
pub struct Action2;
impl InputAction for Action2 {}
pub struct Action3;
impl InputAction for Action3 {}

// These are generic movement commands
pub struct ForwardAction;
impl InputAction for ForwardAction {}
pub struct BackwardAction;
impl InputAction for BackwardAction {}
pub struct LeftAction;
impl InputAction for LeftAction {}
pub struct RightAction;
impl InputAction for RightAction {}

// These exist for developing without using mouse controls
// or for finer movement
pub struct RotateUpAction;
impl InputAction for RotateUpAction {}
pub struct RotateDownAction;
impl InputAction for RotateDownAction {}
pub struct RotateLeftAction;
impl InputAction for RotateLeftAction {}
pub struct RotateRightAction;
impl InputAction for RotateRightAction {}
