use penrose::draw::{Color, DrawContext};

pub enum Align {
    Left,
    Center,
    Right,
}

pub struct WidgetStyle {
    pub x_padding: usize,
    pub y_offset: isize,
    pub fg: Color,
    pub bg: Color,
    pub text_size: usize,
}

pub trait BarWidget {
    /// Render the widget to the status bar.
    ///
    /// Returns the width of the widget.
    ///
    /// # Errors
    ///
    /// Returns an error if the widget fails to draw.
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        font: &str,
        align: Align,
        offset: usize,
        bar_width: usize,
        bar_height: usize,
    ) -> penrose::draw::Result<usize>;

    /// The width required by this widget.
    ///
    /// # Errors
    ///
    /// Returns an error if the widget fails to calculate its extent.
    fn current_width(
        &mut self,
        ctx: &mut dyn DrawContext,
        font: &str,
    ) -> penrose::draw::Result<usize>;
}
