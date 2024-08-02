use accesskit::Role;
use masonry::{
    kurbo::RoundedRect, vello::Scene, AccessCtx, AccessEvent, Action, Affine, BoxConstraints,
    EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, PointerButton, PointerEvent, Size,
    StatusChange, TextEvent, WidgetId,
};
use smallvec::SmallVec;
use xilem::{
    core::{Message, MessageResult, ViewId},
    Color, Pod, ViewCtx,
};

use crate::Player;

pub fn tile(coord: usize, content: Option<Player>, is_playable: bool) -> Tile {
    Tile {
        coord,
        content,
        is_playable,
    }
}

pub struct Tile {
    coord: usize,
    content: Option<Player>,
    is_playable: bool,
}

impl<A> xilem::core::View<crate::Ultimate, A, ViewCtx> for Tile {
    type Element = Pod<TileWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        ctx.with_leaf_action_widget(|_| {
            Pod::new(TileWidget {
                content: self.content,
                // everything is playable at the beginning
                // we will disable the whole board when it's the remote player's turn
                is_playable: true,
            })
        })
    }

    fn rebuild<'el>(
        &self,
        prev: &Self,
        _: &mut Self::ViewState,
        _: &mut ViewCtx,
        mut e: xilem::core::Mut<'el, Self::Element>,
    ) -> xilem::core::Mut<'el, Self::Element> {
        assert_eq!(self.coord, prev.coord, "what za fak");
        if self.content != prev.content {
            e.widget.content = self.content;
            e.ctx.request_paint();
        }
        if self.is_playable != prev.is_playable {
            e.widget.is_playable = self.is_playable;
            e.ctx.set_disabled(!self.is_playable);
            e.ctx.request_paint();
        }
        e
    }

    fn teardown(
        &self,
        _: &mut Self::ViewState,
        _: &mut ViewCtx,
        _: xilem::core::Mut<'_, Self::Element>,
    ) {
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        id: &[ViewId],
        message: Box<dyn Message>,
        ult: &mut crate::Ultimate,
    ) -> MessageResult<A, Box<dyn Message>> {
        assert_eq!(id, &[]);
        let action = message.downcast::<Action>().unwrap();
        let Action::ButtonPressed(button) = *action else {
            panic!()
        };
        if button == PointerButton::Primary && self.is_playable {
            ult.make_move(self.coord);
            return MessageResult::RequestRebuild;
        }
        return MessageResult::Nop;
    }
}

pub struct TileWidget {
    content: Option<Player>,
    is_playable: bool,
}

impl masonry::Widget for TileWidget {
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::PointerDown(_, _) = event {
            ctx.set_active(true);
        }

        if let PointerEvent::PointerUp(button, _) = event {
            if *button == PointerButton::Primary && ctx.is_active() && !ctx.is_disabled() {
                ctx.submit_action(Action::ButtonPressed(*button));
                ctx.set_active(false);
            }
        }
    }

    fn on_text_event(&mut self, _: &mut EventCtx, _: &TextEvent) {}

    fn on_access_event(&mut self, _: &mut EventCtx, _: &AccessEvent) {}

    fn on_status_change(&mut self, ctx: &mut LifeCycleCtx, event: &StatusChange) {
        if let StatusChange::HotChanged(_) = event {
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints) -> Size {
        let size = (24., 24.);
        assert!(bc.contains(size));
        size.into()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let (w, h) = ctx.size().into();
        let colour = if ctx.is_disabled() {
            Color::rgb8(26, 26, 26)
        } else if ctx.is_hot() {
            Color::rgb8(80, 80, 80)
        } else {
            Color::rgb8(40, 40, 40)
        };
        let rect = RoundedRect::new(0., 0., w, h, 2.);
        scene.fill(
            masonry::vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            colour,
            None,
            &rect,
        );

        if self.content == Some(Player::Nought) {
            let nought = masonry::kurbo::Circle::new((12., 12.), 8.);
            scene.stroke(
                &masonry::kurbo::Stroke::new(4.),
                Affine::IDENTITY,
                Color::TURQUOISE,
                None,
                &nought,
            );
        } else if self.content == Some(Player::Cross) {
            let mut cross = masonry::kurbo::BezPath::new();
            cross.move_to((4., 4.));
            cross.line_to((20., 20.));
            cross.close_path();
            cross.move_to((4., 20.));
            cross.line_to((20., 4.));
            cross.close_path();
            scene.stroke(
                &masonry::kurbo::Stroke::new(4.),
                Affine::IDENTITY,
                Color::ORANGE_RED,
                None,
                &cross,
            );
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Unknown
    }

    fn accessibility(&mut self, _: &mut AccessCtx) {}

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        SmallVec::new_const()
    }
}
