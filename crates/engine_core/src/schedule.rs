use std::sync::Arc;

use frosty_alloc::AllocId;

use crate::{query::RawQuery, system::SystemInterface, Spawner};

/*
 * A schedule determines when each update or query gets called
 * and also ensures that they work properly.
 *
 * ex:
 *    the rendering system can't update until after all other
 *    frame updates are finished and fixed-frame updates are
 *    paused
 */

pub(crate) enum NextSystem<'a> {
    System(&'a SystemNode),
    Wait,
    Finished,
}

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

#[derive(Clone)]
pub(crate) struct SystemNodeRaw {
    system: Arc<dyn SystemInterface + 'static>,
    // children nodes
    // index into [Schedule].systems
    deps: Arc<[usize]>,
    // for tracking when ready to start
    waiting_on: u32,
    depends_on: u32,
}

impl SystemNodeRaw {
    pub fn alloc_id(&self) -> AllocId {
        self.system.alloc_id()
    }

    pub fn get_system(&self) -> Arc<dyn SystemInterface> {
        self.system.clone()
    }
}

#[derive(Clone)]
pub(crate) struct SystemNode {
    raw: SystemNodeRaw,
    query: *mut RawQuery,
}

impl SystemNode {
    pub fn alloc_id(&self) -> AllocId {
        self.raw.system.alloc_id()
    }

    pub fn get_system(&self) -> Arc<dyn SystemInterface> {
        self.raw.system.clone()
    }

    pub(crate) fn get_raw(&self) -> SystemNodeRaw {
        self.raw.clone()
    }
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
    ready_systems: Vec<usize>,
    itered_through: usize,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            ready_systems: Vec::new(),
            itered_through: 0,
        }
    }

    // Only fails if the system interop is not ergis
    pub fn add_system<S: SystemInterface + 'static>(
        &mut self,
        system: S,
        alloc: &mut Spawner,
    ) -> Option<()> {
        let query = alloc.get_raw_query(&system.alloc_id())? as *mut RawQuery;
        let node = SystemNode {
            raw: SystemNodeRaw {
                system: Arc::new(system),
                deps: Arc::new([]),
                waiting_on: 0,
                depends_on: 0,
            },
            query,
        };
        self.systems.push(node);
        Some(())
    }

    // load all root [System]s into (ready_systems)
    pub fn prep_systems(&mut self) {
        self.itered_through = 0;
        for (i, s) in self.systems.iter_mut().enumerate() {
            s.raw.waiting_on = s.raw.depends_on;
            if s.raw.waiting_on > 0 {
                continue;
            }
            self.ready_systems.push(i);
        }
    }

    // get the next ready system
    pub fn next<'a>(&'a mut self) -> NextSystem<'a> {
        match (
            self.ready_systems.pop(),
            self.itered_through == self.systems.len(),
        ) {
            (_, true) => NextSystem::Finished,
            (None, false) => NextSystem::Wait,
            (Some(u), false) => NextSystem::System(&self.systems[u]),
        }
    }

    // resets a node for next cycle and adds its children
    // to the ready_systems list
    pub fn return_node(&mut self, node: SystemNodeRaw) {
        self.itered_through += 1;
        let mut i = 0;
        while i < node.deps.len() {
            let dep = &mut self.systems[i];
            dep.raw.waiting_on -= 1;
            if dep.raw.waiting_on == 0 {
                self.ready_systems.push(i);
            }
            i += 1;
        }
    }
}
