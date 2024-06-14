/*
 * A schedule determines when each update or query gets called
 * and also ensures that they work properly.
 *
 * ex:
 *    the rendering system can't update until after all other
 *    frame updates are finished and fixed-frame updates are
 *    paused
 */
pub(crate) struct Schedule {}

impl Schedule {
    pub fn new() -> Self {
        Self {}
    }
}
