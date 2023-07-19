//! A modal for showing elements as an overlay on top of another.
//!
//! *This API requires the following crate features to be activated: modal*
use iced_widget::core::{
    self, event, keyboard, layout,
    mouse::{self, Cursor},
    overlay, renderer, touch,
    widget::Tree,
    Clipboard, Color, Element, Event, Layout, Overlay, Point, Rectangle, Shell, Size, Vector,
};

use crate::style::modal::StyleSheet;

/// The overlay of the modal.
#[allow(missing_debug_implementations)]
pub struct ModalOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The state of the [`ModalOverlay`](ModalOverlay).
    state: &'a mut Tree,
    /// The content of the [`ModalOverlay`](ModalOverlay).
    content: Element<'a, Message, Renderer>,
    /// The optional message that will be send when the user clicks on the backdrop.
    backdrop: Option<Message>,
    /// The optional message that will be send when the ESC key was pressed.
    esc: Option<Message>,
    /// The style of the [`ModalOverlay`](ModalOverlay).
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> ModalOverlay<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`ModalOverlay`](ModalOverlay).
    pub fn new<C>(
        state: &'a mut Tree,
        content: C,
        backdrop: Option<Message>,
        esc: Option<Message>,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self
    where
        C: Into<Element<'a, Message, Renderer>>,
    {
        ModalOverlay {
            state,
            content: content.into(),
            backdrop,
            esc,
            style,
        }
    }

    /// Turn this [`ModalOverlay`] into an overlay
    /// [`Element`](iced_native::overlay::Element).
    pub fn overlay(self, position: Point) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(position, Box::new(self))
    }
}

impl<'a, Message, Renderer> Overlay<Message, Renderer> for ModalOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn layout(&self, renderer: &Renderer, bounds: Size, position: Point) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let mut content = self.content.as_widget().layout(renderer, &limits);

        // Center position
        let max_size = limits.max();
        let container_half_width = max_size.width / 2.0;
        let container_half_height = max_size.height / 2.0;
        let content_half_width = content.bounds().width / 2.0;
        let content_half_height = content.bounds().height / 2.0;

        let position = position
            + Vector::new(
                container_half_width - content_half_width,
                container_half_height - content_half_height,
            );

        content.move_to(position);

        layout::Node::with_children(max_size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        let viewport = layout.bounds();
        // TODO clean this up
        let esc_status = self
            .esc
            .as_ref()
            .map_or(event::Status::Ignored, |esc| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                    if key_code == keyboard::KeyCode::Escape {
                        shell.publish(esc.to_owned());
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                _ => event::Status::Ignored,
            });

        let backdrop_status = self.backdrop.as_ref().zip(layout.children().next()).map_or(
            event::Status::Ignored,
            |(backdrop, layout)| match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if layout
                        .bounds()
                        .contains(cursor.position().unwrap_or_default())
                    {
                        event::Status::Ignored
                    } else {
                        shell.publish(backdrop.to_owned());
                        event::Status::Captured
                    }
                }
                _ => event::Status::Ignored,
            },
        );

        match esc_status.merge(backdrop_status) {
            event::Status::Ignored => self.content.as_widget_mut().on_event(
                self.state,
                event,
                layout
                    .children()
                    .next()
                    .expect("Native: Layout should have a content layout."),
                cursor,
                renderer,
                clipboard,
                shell,
                &viewport,
            ),
            event::Status::Captured => event::Status::Captured,
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.state,
            layout
                .children()
                .next()
                .expect("Native: Layout should have a content layout."),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let bounds = layout.bounds();

        let style_sheet = theme.active(self.style);

        // Background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: (0.0).into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            style_sheet.background,
        );

        let content_layout = layout
            .children()
            .next()
            .expect("Native: Layout should have a content layout.");

        // Modal
        self.content.as_widget().draw(
            self.state,
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            &bounds,
        );
    }
}
