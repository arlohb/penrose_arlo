use penrose::draw::DrawContext;

use crate::Align;

pub trait BarWidget {
    /// Render the widget to the status bar.
    ///
    /// # Errors
    ///
    /// Returns an error if the widget fails to draw.
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        align: Align,
        bar_width: f64,
        bar_height: f64,
    ) -> penrose::draw::Result<()>;

    /// The width and height required by this widget.
    ///
    /// # Errors
    ///
    /// Returns an error if the widget fails to calculate its extent.
    fn current_extent(&mut self, ctx: &mut dyn DrawContext) -> penrose::draw::Result<(f64, f64)>;
}
