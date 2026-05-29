use frust::{
    EventResult, FocusState, InputPolicy, Layer, UiEvent, View, ViewId, ViewNode, ViewTree,
    render_tree,
};
use ratatui::{Frame, Terminal, backend::TestBackend, layout::Rect};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {}

struct OrderedView {
    id: ViewId,
    layer: Layer,
    z: i32,
    ch: char,
}

impl OrderedView {
    fn new(id: &'static str, layer: Layer, z: i32, ch: char) -> Self {
        Self {
            id: ViewId::new(id),
            layer,
            z,
            ch,
        }
    }
}

impl View<(), Msg> for OrderedView {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        InputPolicy::HitTest
    }

    fn layer(&self) -> Layer {
        self.layer
    }

    fn z_offset(&self) -> i32 {
        self.z
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, _state: &()) {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = frame.buffer_mut().cell_mut((x, y)) {
                    cell.set_char(self.ch);
                }
            }
        }
    }

    fn handle_event(
        &self,
        _event: &UiEvent,
        _area: Rect,
        _state: &(),
        _focus: &FocusState,
    ) -> EventResult<Msg> {
        EventResult::Ignored
    }
}

#[test]
fn render_order_is_layer_then_z_then_insertion_and_tooltip_is_last() {
    let tree = ViewTree::new(
        frust::root(Rect::new(0, 0, 10, 5))
            .child(ViewNode::new(
                OrderedView::new("base-a", Layer::Base, 0, 'a'),
                Rect::new(0, 0, 1, 1),
            ))
            .child(ViewNode::new(
                OrderedView::new("base-b", Layer::Base, 0, 'b'),
                Rect::new(0, 0, 1, 1),
            ))
            .child(ViewNode::new(
                OrderedView::new("modal", Layer::Modal, 0, 'm'),
                Rect::new(0, 0, 1, 1),
            ))
            .child(ViewNode::new(
                OrderedView::new("overlay", Layer::Overlay, 0, 'o'),
                Rect::new(0, 0, 1, 1),
            ))
            .child(ViewNode::new(
                OrderedView::new("tooltip", Layer::Tooltip, 0, 't'),
                Rect::new(0, 0, 1, 1),
            )),
    );

    let ids = tree
        .render_order()
        .into_iter()
        .map(|node| node.id.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec!["root", "base-a", "base-b", "overlay", "modal", "tooltip"]
    );

    let mut terminal = Terminal::new(TestBackend::new(10, 5)).unwrap();
    terminal
        .draw(|frame| render_tree(&tree, frame, &()))
        .unwrap();
    let symbol = terminal.backend().buffer().cell((0, 0)).unwrap().symbol();
    assert_eq!(symbol, "t");
}
