use std::{cell::Cell, ops::Range, rc::Rc};

use gpui::{
    point, quad, AppContext, Bounds, ContentMask, Corners, Edges, EntityId, FocusHandle, Hitbox,
    Hsla, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ScrollWheelEvent, Style,
    UniformListScrollHandle,
};
use ui::{prelude::*, px, relative, IntoElement};

pub fn render_vertical_scrollbar(
    parent_id: EntityId,
    parent_focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle::new(),
    scrollbar_drag_thumb_offset: Rc<Cell<Option<f32>>>,
    cx: &mut AppContext,
) -> Option<Stateful<Div>> {
    if !self.show_scrollbar || !Self::should_show_scrollbar(cx) {
        return None;
    }
    let scroll_handle = scroll_handle.0.borrow();
    let total_list_length = scroll_handle
        .last_item_size
        .filter(|_| scrollbar_drag_thumb_offset.get().is_some())?
        .contents
        .height
        .0 as f64;
    let current_offset = scroll_handle.base_handle.offset().y.0.min(0.).abs() as f64;
    let mut percentage = current_offset / total_list_length;
    let end_offset = (current_offset + scroll_handle.base_handle.bounds().size.height.0 as f64)
        / total_list_length;
    // Uniform scroll handle might briefly report an offset greater than the length of a list;
    // in such case we'll adjust the starting offset as well to keep the scrollbar thumb length stable.
    let overshoot = (end_offset - 1.).clamp(0., 1.);
    if overshoot > 0. {
        percentage -= overshoot;
    }
    const MINIMUM_SCROLLBAR_PERCENTAGE_HEIGHT: f64 = 0.005;
    if percentage + MINIMUM_SCROLLBAR_PERCENTAGE_HEIGHT > 1.0 || end_offset > total_list_length {
        return None;
    }
    if total_list_length < scroll_handle.base_handle.bounds().size.height.0 as f64 {
        return None;
    }
    let end_offset = end_offset.clamp(percentage + MINIMUM_SCROLLBAR_PERCENTAGE_HEIGHT, 1.);
    Some(
        div()
            .occlude()
            .id("generic-vertical-scroll")
            .on_mouse_move(cx.listener(|_, _, cx| {
                cx.notify();
                cx.stop_propagation()
            }))
            .on_hover(|_, cx| {
                cx.stop_propagation();
            })
            .on_any_mouse_down(|_, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, cx| {
                    if scrollbar_drag_thumb_offset.get().is_none()
                        && !parent_focus_handle.contains_focused(cx)
                    {
                        this.hide_scrollbar(cx);
                        cx.notify();
                    }

                    cx.stop_propagation();
                }),
            )
            .on_scroll_wheel(cx.listener(|_, _, cx| {
                cx.notify();
            }))
            .h_full()
            .absolute()
            .right_1()
            .top_1()
            .bottom_1()
            .w(px(12.))
            .cursor_default()
            .child(Scrollbar::vertical(
                percentage as f32..end_offset as f32,
                scroll_handle,
                scrollbar_drag_thumb_offset,
                parent_id,
            )),
    )
}

pub fn render_horizontal_scrollbar(
    parent_id: EntityId,
    parent_focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle::new(),
    scrollbar_drag_thumb_offset: Rc<Cell<Option<f32>>>,
    cx: &mut AppContext,
) -> Option<Stateful<Div>> {
    if !self.show_scrollbar || !Self::should_show_scrollbar(cx) || self.width.is_none() {
        return None;
    }
    let scroll_handle = scroll_handle.0.borrow();
    let longest_item_width = scroll_handle
        .last_item_size
        .filter(|_| scrollbar_drag_thumb_offset.get().is_some())
        .filter(|size| size.contents.width > size.item.width)?
        .contents
        .width
        .0 as f64;
    let current_offset = scroll_handle.base_handle.offset().x.0.min(0.).abs() as f64;
    let mut percentage = current_offset / longest_item_width;
    let end_offset = (current_offset + scroll_handle.base_handle.bounds().size.width.0 as f64)
        / longest_item_width;
    // Uniform scroll handle might briefly report an offset greater than the length of a list;
    // in such case we'll adjust the starting offset as well to keep the scrollbar thumb length stable.
    let overshoot = (end_offset - 1.).clamp(0., 1.);
    if overshoot > 0. {
        percentage -= overshoot;
    }
    const MINIMUM_SCROLLBAR_PERCENTAGE_WIDTH: f64 = 0.005;
    if percentage + MINIMUM_SCROLLBAR_PERCENTAGE_WIDTH > 1.0 || end_offset > longest_item_width {
        return None;
    }
    if longest_item_width < scroll_handle.base_handle.bounds().size.width.0 as f64 {
        return None;
    }
    let end_offset = end_offset.clamp(percentage + MINIMUM_SCROLLBAR_PERCENTAGE_WIDTH, 1.);
    Some(
        div()
            .occlude()
            .id("generic-horizontal-scroll")
            .on_mouse_move(cx.listener(|_, _, cx| {
                cx.notify();
                cx.stop_propagation()
            }))
            .on_hover(|_, cx| {
                cx.stop_propagation();
            })
            .on_any_mouse_down(|_, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, cx| {
                    if scrollbar_drag_thumb_offset.get().is_none()
                        && !parent_focus_handle.contains_focused(cx)
                    {
                        this.hide_scrollbar(cx);
                        cx.notify();
                    }

                    cx.stop_propagation();
                }),
            )
            .on_scroll_wheel(cx.listener(|_, _, cx| {
                cx.notify();
            }))
            .w_full()
            .absolute()
            .right_1()
            .left_1()
            .bottom_1()
            .h(px(12.))
            .cursor_default()
            .child(Scrollbar::horizontal(
                percentage as f32..end_offset as f32,
                scroll_handle.clone(),
                scrollbar_drag_thumb_offset.clone(),
                parent_id,
            )),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarKind {
    Horizontal,
    Vertical,
}

pub struct Scrollbar {
    thumb: Range<f32>,
    scroll: UniformListScrollHandle,
    // If Some(), there's an active drag, offset by percentage from the top of thumb.
    scrollbar_drag_state: Rc<Cell<Option<f32>>>,
    kind: ScrollbarKind,
    parent_id: EntityId,
}

impl Scrollbar {
    pub fn vertical(
        thumb: Range<f32>,
        scroll: UniformListScrollHandle,
        scrollbar_drag_state: Rc<Cell<Option<f32>>>,
        parent_id: EntityId,
    ) -> Self {
        Self {
            thumb,
            scroll,
            scrollbar_drag_state,
            kind: ScrollbarKind::Vertical,
            parent_id,
        }
    }

    pub fn horizontal(
        thumb: Range<f32>,
        scroll: UniformListScrollHandle,
        scrollbar_drag_state: Rc<Cell<Option<f32>>>,
        parent_id: EntityId,
    ) -> Self {
        Self {
            thumb,
            scroll,
            scrollbar_drag_state,
            kind: ScrollbarKind::Horizontal,
            parent_id,
        }
    }
}

impl gpui::Element for Scrollbar {
    type RequestLayoutState = ();

    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        cx: &mut ui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 1.;
        style.flex_shrink = 1.;
        if self.kind == ScrollbarKind::Vertical {
            style.size.width = px(12.).into();
            style.size.height = relative(1.).into();
        } else {
            style.size.width = relative(1.).into();
            style.size.height = px(12.).into();
        }

        (cx.request_layout(style, None), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: Bounds<ui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        cx: &mut ui::WindowContext,
    ) -> Self::PrepaintState {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            cx.insert_hitbox(bounds, false)
        })
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: Bounds<ui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        cx: &mut ui::WindowContext,
    ) {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            let colors = cx.theme().colors();
            let thumb_background = colors.scrollbar_thumb_background;
            let is_vertical = self.kind == ScrollbarKind::Vertical;
            let extra_padding = px(5.0);
            let padded_bounds = if is_vertical {
                Bounds::from_corners(
                    bounds.origin + point(Pixels::ZERO, extra_padding),
                    bounds.lower_right() - point(Pixels::ZERO, extra_padding * 3),
                )
            } else {
                Bounds::from_corners(
                    bounds.origin + point(extra_padding, Pixels::ZERO),
                    bounds.lower_right() - point(extra_padding * 3, Pixels::ZERO),
                )
            };

            let mut thumb_bounds = if is_vertical {
                let thumb_offset = self.thumb.start * padded_bounds.size.height;
                let thumb_end = self.thumb.end * padded_bounds.size.height;
                let thumb_upper_left = point(
                    padded_bounds.origin.x,
                    padded_bounds.origin.y + thumb_offset,
                );
                let thumb_lower_right = point(
                    padded_bounds.origin.x + padded_bounds.size.width,
                    padded_bounds.origin.y + thumb_end,
                );
                Bounds::from_corners(thumb_upper_left, thumb_lower_right)
            } else {
                let thumb_offset = self.thumb.start * padded_bounds.size.width;
                let thumb_end = self.thumb.end * padded_bounds.size.width;
                let thumb_upper_left = point(
                    padded_bounds.origin.x + thumb_offset,
                    padded_bounds.origin.y,
                );
                let thumb_lower_right = point(
                    padded_bounds.origin.x + thumb_end,
                    padded_bounds.origin.y + padded_bounds.size.height,
                );
                Bounds::from_corners(thumb_upper_left, thumb_lower_right)
            };
            let corners = if is_vertical {
                thumb_bounds.size.width /= 1.5;
                Corners::all(thumb_bounds.size.width / 2.0)
            } else {
                thumb_bounds.size.height /= 1.5;
                Corners::all(thumb_bounds.size.height / 2.0)
            };
            cx.paint_quad(quad(
                thumb_bounds,
                corners,
                thumb_background,
                Edges::default(),
                Hsla::transparent_black(),
            ));

            let scroll = self.scroll.clone();
            let kind = self.kind;
            let thumb_percentage_size = self.thumb.end - self.thumb.start;

            cx.on_mouse_event({
                let scroll = self.scroll.clone();
                let is_dragging = self.scrollbar_drag_state.clone();
                move |event: &MouseDownEvent, phase, _cx| {
                    if phase.bubble() && bounds.contains(&event.position) {
                        if !thumb_bounds.contains(&event.position) {
                            let scroll = scroll.0.borrow();
                            if let Some(item_size) = scroll.last_item_size {
                                match kind {
                                    ScrollbarKind::Horizontal => {
                                        let percentage = (event.position.x - bounds.origin.x)
                                            / bounds.size.width;
                                        let max_offset = item_size.contents.width;
                                        let percentage = percentage.min(1. - thumb_percentage_size);
                                        scroll.base_handle.set_offset(point(
                                            -max_offset * percentage,
                                            scroll.base_handle.offset().y,
                                        ));
                                    }
                                    ScrollbarKind::Vertical => {
                                        let percentage = (event.position.y - bounds.origin.y)
                                            / bounds.size.height;
                                        let max_offset = item_size.contents.height;
                                        let percentage = percentage.min(1. - thumb_percentage_size);
                                        scroll.base_handle.set_offset(point(
                                            scroll.base_handle.offset().x,
                                            -max_offset * percentage,
                                        ));
                                    }
                                }
                            }
                        } else {
                            let thumb_offset = if is_vertical {
                                (event.position.y - thumb_bounds.origin.y) / bounds.size.height
                            } else {
                                (event.position.x - thumb_bounds.origin.x) / bounds.size.width
                            };
                            is_dragging.set(Some(thumb_offset));
                        }
                    }
                }
            });
            cx.on_mouse_event({
                let scroll = self.scroll.clone();
                move |event: &ScrollWheelEvent, phase, cx| {
                    if phase.bubble() && bounds.contains(&event.position) {
                        let scroll = scroll.0.borrow_mut();
                        let current_offset = scroll.base_handle.offset();

                        scroll
                            .base_handle
                            .set_offset(current_offset + event.delta.pixel_delta(cx.line_height()));
                    }
                }
            });
            let drag_state = self.scrollbar_drag_state.clone();
            let view_id = self.parent_id;
            let kind = self.kind;
            cx.on_mouse_event(move |event: &MouseMoveEvent, _, cx| {
                if let Some(drag_state) = drag_state.get().filter(|_| event.dragging()) {
                    let scroll = scroll.0.borrow();
                    if let Some(item_size) = scroll.last_item_size {
                        match kind {
                            ScrollbarKind::Horizontal => {
                                let max_offset = item_size.contents.width;
                                let percentage = (event.position.x - bounds.origin.x)
                                    / bounds.size.width
                                    - drag_state;

                                let percentage = percentage.min(1. - thumb_percentage_size);
                                scroll.base_handle.set_offset(point(
                                    -max_offset * percentage,
                                    scroll.base_handle.offset().y,
                                ));
                            }
                            ScrollbarKind::Vertical => {
                                let max_offset = item_size.contents.height;
                                let percentage = (event.position.y - bounds.origin.y)
                                    / bounds.size.height
                                    - drag_state;

                                let percentage = percentage.min(1. - thumb_percentage_size);
                                scroll.base_handle.set_offset(point(
                                    scroll.base_handle.offset().x,
                                    -max_offset * percentage,
                                ));
                            }
                        };

                        cx.notify(view_id);
                    }
                } else {
                    drag_state.set(None);
                }
            });
            let is_dragging = self.scrollbar_drag_state.clone();
            cx.on_mouse_event(move |_event: &MouseUpEvent, phase, cx| {
                if phase.bubble() {
                    is_dragging.set(None);
                    cx.notify(view_id);
                }
            });
        })
    }
}

impl IntoElement for Scrollbar {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
