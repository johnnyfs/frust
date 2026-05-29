//! Reusable terminal UI composition toolkit.
//!
//! `tui` keeps terminal UI composition, z-ordering, input routing, focus,
//! mouse capture, and rendering traversal explicit. Applications keep durable
//! state and domain semantics.

pub mod compose;
pub mod event;
pub mod focus;
pub mod layer;
pub mod render;
pub mod route;
pub mod tree;
pub mod view;
pub mod widgets;

pub use compose::root;
pub use event::{MouseButton, MouseEvent, MouseKind, UiEvent};
pub use focus::{FocusState, FocusUpdate};
pub use layer::Layer;
pub use render::render_tree;
pub use route::{RouteOutcome, Router, route_event};
pub use tree::{NodeRef, Point, ViewId, ViewNode, ViewTree};
pub use view::{EventResult, InputPolicy, View};
