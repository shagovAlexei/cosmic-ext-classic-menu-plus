// SPDX-License-Identifier: GPL-3.0-only

use cosmic::Element;
use cosmic::iced::core::layout::{Layout, Limits, Node};
use cosmic::iced::core::widget::{Operation, Tree, Widget};
use cosmic::iced::core::{Clipboard, Shell, mouse, overlay, renderer};
use cosmic::iced::{Event, Length, Rectangle, Size, Vector};

/// Wraps `cosmic::widget::context_menu` placed inside a `scrollable`.
///
/// A scrollable hands its content a cursor and layout in content coordinates
/// (shifted by the scroll offset), but `context_menu` anchors its popup at the
/// cursor position as if it were a window coordinate, so the menu appears
/// `scroll_offset` pixels too low. This widget converts both back to window
/// coordinates for `update`, the only place the popup is created.
pub struct ScrollAwareContextMenu<'a, Message> {
    inner: Element<'a, Message>,
    scroll_offset: f32,
    /// Node from the last `layout` call, positioned at the origin.
    /// `Layout` cannot be shifted without the node it came from.
    node: Node,
}

impl<'a, Message> ScrollAwareContextMenu<'a, Message> {
    pub fn new(inner: impl Into<Element<'a, Message>>, scroll_offset: f32) -> Self {
        Self {
            inner: inner.into(),
            scroll_offset,
            node: Node::default(),
        }
    }
}

impl<Message: Clone + 'static> Widget<Message, cosmic::Theme, cosmic::Renderer>
    for ScrollAwareContextMenu<'_, Message>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.inner.as_widget())]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.inner));
    }

    fn size(&self) -> Size<Length> {
        self.inner.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &cosmic::Renderer, limits: &Limits) -> Node {
        let node = self
            .inner
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        self.node = node.clone();
        node
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut cosmic::Renderer,
        theme: &cosmic::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.inner.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &cosmic::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        self.inner
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &cosmic::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let shift = Vector::new(0.0, -self.scroll_offset);
        let position = layout.position();
        let window_layout = Layout::with_offset(
            Vector::new(position.x, position.y) + shift,
            &self.node,
        );
        let window_cursor = match cursor {
            mouse::Cursor::Available(point) => mouse::Cursor::Available(point + shift),
            other => other,
        };

        self.inner.as_widget_mut().update(
            &mut tree.children[0],
            event,
            window_layout,
            window_cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &cosmic::Renderer,
    ) -> mouse::Interaction {
        self.inner.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &cosmic::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, cosmic::Theme, cosmic::Renderer>> {
        self.inner.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'static> From<ScrollAwareContextMenu<'a, Message>> for Element<'a, Message> {
    fn from(widget: ScrollAwareContextMenu<'a, Message>) -> Self {
        Self::new(widget)
    }
}
