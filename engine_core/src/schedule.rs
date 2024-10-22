use std::ptr::NonNull;

use crate::system::SystemInterface;

/*
 * A schedule determines when each update or query gets called
 * and also ensures that they work properly.
 *
 * ex:
 *    the rendering system can't update until after all other
 *    frame updates are finished and fixed-frame updates are
 *    paused
 */

// How to make sure systems run in proper order?
//
// Given the following [System]s and their dependencies:
//      A
//      B -> A
//      C -> F
//      D
//      E -> C
//      F -> A
// We can construct the following trees
//      A           D
//     / \
//     B  F
//        |
//        C
//        |
//        E
// It is clear that the trees are generic. In the example, the max
// children a node has is only 2, but there is no reason why it couldn't
// have 3, 4, or even 50 children.
//
// Now a more complicated example
//      A
//      B
//      C -> A, B
//
//    A   B
//     \ /
//      C
//
// It is clear from this a node isn't simply owned by its parents.
// While a [System] may depend on other [System]s, it also exists
// completely seperate from them.

struct SystemNode {
    system: Box<dyn SystemInterface>,
    // children nodes
    // index into [Schedule].systems
    deps: Vec<usize>,
    // for tracking when ready to start
    waiting_on: u32,
    depends_on: u32,
}

// Using a [System] tree like shown above,
// (systems) can be constructed by appending a row at a time.
// ex:
//          A           D
//          |          / \
//          B         E.  F
//          |         |
//          C         G
// the rows would be:
//     (A, D), (B, E, F), (C, G)
// and (systems) would come out as
//     [A, D, B, E, F, C, G]
// and the roots can easily be found as all Systems until one that's dependent is found
//     A is not dependent
//     D is not dependent (A)
//     B is depdendent    (A, D)
//     roots: (A, D)

pub(crate) struct Schedule {
    systems: Vec<SystemNode>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn next<'a>(&self) -> &'a dyn SystemInterface {
        todo!()
    }
}
