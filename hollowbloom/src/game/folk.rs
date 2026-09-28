//! The people of Bramblewick.

use super::Io;
use super::play::Play;

impl Play {
    /// Moves the townsfolk about. `active` is false while a menu or fade holds the world.
    pub fn update_folk(&mut self, _dt: f32, _active: bool) {}

    /// Puts the right people in the right places when you arrive somewhere.
    pub fn arrive_folk(&mut self) {}

    /// The request board.
    pub fn open_board(&mut self, _io: &mut Io) {
        self.toast("The board is empty today.", None, 0);
    }
}
