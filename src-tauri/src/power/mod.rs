/// Holds at most one OS sleep-prevention assertion.
///
/// `acquire` while one is held must replace it: create the new assertion
/// first, then release the old one, so there is never a gap.
pub trait Power: Send {
    /// `display == true` keeps the display awake as well as the system.
    fn acquire(&mut self, display: bool) -> Result<(), String>;
    fn release(&mut self);
}
