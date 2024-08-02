use accesskit::Role;
use masonry::{
    vello::Scene, AccessCtx, AccessEvent, BoxConstraints, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, PointerEvent, Size, StatusChange, TextEvent, Widget, WidgetId,
    WidgetPod,
};
use smallvec::SmallVec;
use xilem::{
    core::{Message, Mut, View},
    Pod, ViewCtx, WidgetView,
};

pub fn disable_if<V>(disabled: bool, inner: V) -> DisableIf<V> {
    DisableIf(disabled, inner)
}

pub struct DisableIf<V>(bool, V);

impl<T, A, V> View<T, A, ViewCtx> for DisableIf<V>
where
    V: WidgetView<T, A>,
{
    type Element = Pod<DisableIfWidget>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = self.1.build(ctx);
        let element = DisableIfWidget(self.0, child.inner.boxed());
        (Pod::new(element), child_state)
    }

    fn rebuild<'el>(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'el, Self::Element>,
    ) -> Mut<'el, Self::Element> {
        if self.0 != prev.0 {
            element.widget.0 = self.0;
            element.ctx.set_disabled(self.0);
        }
        {
            let mut child = element.ctx.get_mut(&mut element.widget.1);
            self.1.rebuild(&prev.1, view_state, ctx, child.downcast());
        }
        element
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        let mut child = element.ctx.get_mut(&mut element.widget.1);
        self.1.teardown(view_state, ctx, child.downcast())
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        id_path: &[xilem::core::ViewId],
        message: Box<dyn Message>,
        app_state: &mut T,
    ) -> xilem::core::MessageResult<A, Box<dyn Message>> {
        self.1.message(view_state, id_path, message, app_state)
    }
}

pub struct DisableIfWidget(bool, WidgetPod<Box<dyn Widget>>);

impl Widget for DisableIfWidget {
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        self.1.on_pointer_event(ctx, event)
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        self.1.on_text_event(ctx, event)
    }

    fn on_access_event(&mut self, ctx: &mut EventCtx, event: &AccessEvent) {
        self.1.on_access_event(ctx, event)
    }

    fn on_status_change(&mut self, _: &mut LifeCycleCtx, _: &StatusChange) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        if let LifeCycle::WidgetAdded = event {
            ctx.set_disabled(self.0);
        }
        self.1.lifecycle(ctx, event)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints) -> Size {
        let size = self.1.layout(ctx, bc);
        ctx.place_child(&mut self.1, Point::new(0., 0.));
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        self.1.paint(ctx, scene)
    }

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
    }

    fn accessibility(&mut self, ctx: &mut AccessCtx) {
        self.1.accessibility(ctx)
    }

    fn children_ids(&self) -> SmallVec<[WidgetId; 16]> {
        smallvec::smallvec![self.1.id()]
    }
}
