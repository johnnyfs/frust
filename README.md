# frust

`frust` is a small retained composition layer on top of Ratatui and Crossterm.
It owns terminal UI mechanics that are easy to get subtly wrong: view-tree
composition, rectangle ownership, z-ordering, deterministic render traversal,
event routing, keyboard focus, mouse hit-testing, mouse capture, modal capture,
scrollable primitives, custom rendering hooks, and message emission.

## What This Is

This crate is a reusable UI foundation for applications that already own their
state and semantics. A composed `ViewTree` lasts for one frame, can be rendered,
hit-tested, routed, and inspected in tests, then is rebuilt from app state on the
next frame.

## What This Is Not

This crate is not an application framework. It does not own business state,
persistence, networking, async policy, or the event loop architecture. It does
not hide Ratatui's `Frame`; custom views can draw directly with Ratatui.

## Mental Model

```text
app_state
  -> compose(app_state, terminal_area) -> ViewTree
  -> route_event(event, view_tree, app_state, focus_state) -> RouteOutcome<AppMessage>
  -> app_update(messages + focus_update, app_state)
  -> render(view_tree, app_state, frame)
```

The application owns durable state. Views are mostly stateless renderers over
that state. The router does not mutate app state; it emits messages and an
explicit `FocusUpdate`.

## View Tree And Z-Order

Each `ViewNode` has a `ViewId`, `Rect`, `Layer`, numeric `z_offset`, input
policy, optional children, and a render/event hook through the `View` trait.
Render order is deterministic:

```text
Base -> Overlay -> Modal -> Tooltip
```

Within a layer, `z_offset` sorts first and stable insertion order breaks ties.
Event hit-testing uses the same ordering in reverse so the topmost eligible view
wins.

## Input Routing

Routing order is deterministic:

1. Active `CaptureAll` modal receives the event first.
2. Mouse capture receives drag/move/up events even outside bounds.
3. Keyboard focus receives key events.
4. Mouse events hit-test topmost eligible views.
5. `Bubble` offers the event to parent nodes.
6. A configured router can offer unhandled events to the root.

`InputPolicy` controls participation: `None`, `HitTest`, `Focusable`,
`CaptureKeyboard`, `CaptureMouse`, and `CaptureAll`.

## Focus And Mouse Capture

`FocusState` tracks `keyboard_focus`, `mouse_capture`, `hovered`, and
`active_modal`. Routing returns a `FocusUpdate`; callers decide when to apply it.
Mouse down on a focusable view can update keyboard focus. Mouse down on a
mouse-capturing view can start capture, and mouse up releases it.

## Custom Rendering

Implement `View<S, M>` directly or use `widgets::CustomView`. The render method
receives `&mut ratatui::Frame`, the owned `Rect`, and `&S`, so advanced Ratatui
drawing remains available.

## Primitive Views

Included primitives:

- `Panel`: bordered or unbordered region with optional clearing.
- `Modal`: centered or explicitly placed overlay with capture policy and
  optional close message.
- `ScrollView`: stateless scroll viewport with app-owned scroll offsets and
  scroll messages.
- `Tooltip`: top-layer hover box.
- `Tabs`: tab labels with click and key-selection messages.
- `CellGrid`: canvas-like cell grid with coordinate conversion helpers.
- `CustomView`: direct render/event hook.

## Minimal Example

```rust
use frust::{
    InputPolicy, ViewNode, ViewTree, route_event, render_tree,
    widgets::Panel,
};
use ratatui::layout::Rect;

#[derive(Default)]
struct AppState;

#[derive(Debug, Clone)]
enum Msg {
    SelectPanel,
}

fn compose(area: Rect) -> ViewTree<AppState, Msg> {
    ViewTree::new(
        frust::root(area).child(ViewNode::new(
            Panel::new("main")
                .title("Main")
                .input_policy(InputPolicy::Focusable),
            area,
        )),
    )
}
# fn demo(frame: &mut ratatui::Frame<'_>) {
# let state = AppState;
# let focus = frust::FocusState::default();
let tree = compose(frame.area());
let outcome = route_event(&frust::UiEvent::Tick, &tree, &state, &focus);
render_tree(&tree, frame, &state);
# let _ = outcome;
# }
```

## Design Notes And Limitations

Composition is retained only for the current frame and should be rebuilt from
application state. Modals are high-layer views with `CaptureAll`; they are not a
separate subsystem. Render traversal and input routing are separate operations
that derive from the same tree.

Current limitations: clipping is documented as an intent flag for custom views,
not a hard rendering sandbox; scroll state is app-owned and emitted as messages;
focus traversal policy is intentionally minimal; complex layout systems and async
event-loop policy are left to the application.
