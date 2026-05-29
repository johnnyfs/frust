//! Retained view tree and geometry types.

use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

use ratatui::layout::Rect;

use crate::{InputPolicy, Layer, View};

static NEXT_VIEW_ID: AtomicU64 = AtomicU64::new(1);

/// Stable identifier for a view.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewId(String);

impl ViewId {
    /// Creates an explicit view id.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Creates a generated view id.
    pub fn generated() -> Self {
        let id = NEXT_VIEW_ID.fetch_add(1, Ordering::Relaxed);
        Self(format!("view-{id}"))
    }

    /// Returns the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ViewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ViewId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ViewId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Terminal coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// Column.
    pub x: u16,
    /// Row.
    pub y: u16,
}

impl Point {
    /// Creates a point.
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Returns true when the point is inside a rectangle.
    pub fn is_inside(self, rect: Rect) -> bool {
        self.x >= rect.x
            && self.y >= rect.y
            && self.x < rect.x.saturating_add(rect.width)
            && self.y < rect.y.saturating_add(rect.height)
    }
}

/// A retained view node for one composed frame.
pub struct ViewNode<S, M> {
    view: Box<dyn View<S, M>>,
    rect: Rect,
    children: Vec<ViewNode<S, M>>,
    clip: bool,
}

impl<S: 'static, M: 'static> ViewNode<S, M> {
    /// Creates a node from a view and owned rectangle.
    pub fn new<V>(view: V, rect: Rect) -> Self
    where
        V: View<S, M> + 'static,
    {
        Self {
            view: Box::new(view),
            rect,
            children: Vec::new(),
            clip: false,
        }
    }

    /// Returns this node with a different rectangle.
    pub fn at(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Enables or disables clipping intent for this node.
    ///
    /// Ratatui widgets naturally clip to their render area. Custom render hooks
    /// should honor this flag when they draw outside their area.
    pub fn clipped(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Appends a child node.
    pub fn child(mut self, child: ViewNode<S, M>) -> Self {
        self.children.push(child);
        self
    }

    /// Appends an overlay child. This is an alias for `child` that reads well at
    /// composition sites.
    pub fn overlay(self, child: ViewNode<S, M>) -> Self {
        self.child(child)
    }

    /// Appends a modal child when `show` is true.
    pub fn modal_if(self, show: bool, child: ViewNode<S, M>) -> Self {
        if show { self.child(child) } else { self }
    }

    /// Mutably pushes a child node.
    pub fn push_child(&mut self, child: ViewNode<S, M>) {
        self.children.push(child);
    }

    /// Returns this node's id.
    pub fn id(&self) -> ViewId {
        self.view.id()
    }

    /// Returns this node's rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Returns this node's children.
    pub fn children(&self) -> &[ViewNode<S, M>] {
        &self.children
    }
}

/// A composed view tree for one frame.
pub struct ViewTree<S, M> {
    root: ViewNode<S, M>,
}

impl<S: 'static, M: 'static> ViewTree<S, M> {
    /// Creates a tree from a root node.
    pub fn new(root: ViewNode<S, M>) -> Self {
        Self { root }
    }

    /// Returns the root node.
    pub fn root(&self) -> &ViewNode<S, M> {
        &self.root
    }

    /// Returns flattened node references in stable insertion order.
    pub fn flatten(&self) -> Vec<NodeRef<'_, S, M>> {
        let mut out = Vec::new();
        flatten_node(&self.root, None, &mut out);
        out
    }

    /// Returns nodes in deterministic render order.
    pub fn render_order(&self) -> Vec<NodeRef<'_, S, M>> {
        let mut nodes = self.flatten();
        nodes.sort_by_key(|node| (node.layer, node.z_offset, node.insertion));
        nodes
    }

    /// Returns nodes in deterministic topmost-first event order.
    pub fn event_order(&self) -> Vec<NodeRef<'_, S, M>> {
        let mut nodes = self.flatten();
        nodes.sort_by(|a, b| {
            (b.layer, b.z_offset, b.insertion).cmp(&(a.layer, a.z_offset, a.insertion))
        });
        nodes
    }

    /// Finds a node by id.
    pub fn find(&self, id: &ViewId) -> Option<NodeRef<'_, S, M>> {
        self.flatten().into_iter().find(|node| &node.id == id)
    }

    /// Returns the topmost input-eligible node under a point.
    pub fn hit_test(&self, point: Point) -> Option<NodeRef<'_, S, M>> {
        self.event_order()
            .into_iter()
            .find(|node| node.input_policy.can_hit_test() && point.is_inside(node.rect))
    }
}

fn flatten_node<'a, S: 'static, M: 'static>(
    node: &'a ViewNode<S, M>,
    parent: Option<usize>,
    out: &mut Vec<NodeRef<'a, S, M>>,
) {
    let insertion = out.len();
    out.push(NodeRef {
        index: insertion,
        parent,
        id: node.view.id(),
        rect: node.rect,
        layer: node.view.layer(),
        z_offset: node.view.z_offset(),
        input_policy: node.view.input_policy(),
        insertion,
        clip: node.clip,
        view: node.view.as_ref(),
    });

    for child in &node.children {
        flatten_node(child, Some(insertion), out);
    }
}

/// A flattened reference to a view node.
pub struct NodeRef<'a, S, M> {
    /// Stable flat index.
    pub index: usize,
    /// Parent flat index, if present.
    pub parent: Option<usize>,
    /// View id.
    pub id: ViewId,
    /// Owned view rectangle.
    pub rect: Rect,
    /// Semantic layer.
    pub layer: Layer,
    /// Numeric z offset within the semantic layer.
    pub z_offset: i32,
    /// Input participation.
    pub input_policy: InputPolicy,
    /// Stable insertion order.
    pub insertion: usize,
    /// Whether custom rendering should clip to area.
    pub clip: bool,
    /// View implementation.
    pub view: &'a dyn View<S, M>,
}

impl<S, M> Clone for NodeRef<'_, S, M> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            parent: self.parent,
            id: self.id.clone(),
            rect: self.rect,
            layer: self.layer,
            z_offset: self.z_offset,
            input_policy: self.input_policy,
            insertion: self.insertion,
            clip: self.clip,
            view: self.view,
        }
    }
}
