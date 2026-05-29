//! Layer and z-order primitives.

/// Semantic render and input layer.
///
/// Render order is ascending (`Base` first, `Tooltip` last). Event hit testing
/// checks the same ordering in reverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Layer {
    /// Ordinary page content.
    Base,
    /// Popovers, menus, and panels above base content.
    Overlay,
    /// Modal prompts and dialogs.
    Modal,
    /// Ephemeral annotations above everything else.
    Tooltip,
}
