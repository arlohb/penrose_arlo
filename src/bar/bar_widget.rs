use penrose::draw::DrawContext;

pub enum Align {
    Left,
    Center,
    Right,
}

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
        bar_width: usize,
        bar_height: usize,
    ) -> penrose::draw::Result<()>;

    /// The width required by this widget.
    ///
    /// # Errors
    ///
    /// Returns an error if the widget fails to calculate its extent.
    fn current_width(&mut self, ctx: &mut dyn DrawContext) -> penrose::draw::Result<usize>;
}
