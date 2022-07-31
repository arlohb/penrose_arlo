use penrose::draw::{DrawContext, TextStyle};

use crate::BarWidget;

pub enum Align {
    Left,
    Center,
    Right,
}

pub struct ReactiveText {
    text: Box<dyn FnMut() -> Option<String>>,
    text_style: TextStyle,
}

impl ReactiveText {
    pub fn new(text: impl FnMut() -> Option<String> + 'static, text_style: TextStyle) -> Box<Self> {
        Box::new(Self {
            text: Box::new(text),
            text_style,
        })
    }

    pub fn text(&mut self) -> String {
        (self.text)().unwrap_or_else(|| "".to_string())
    }
}

impl BarWidget for ReactiveText {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        align: Align,
        bar_width: f64,
        _bar_height: f64,
    ) -> penrose::draw::Result<()> {
        let extent = self.current_extent(ctx)?;

        ctx.font(&self.text_style.font, self.text_style.point_size)?;
        ctx.color(&self.text_style.fg);

        ctx.set_x_offset(match align {
            Align::Left => 0.,
            Align::Center => (bar_width - extent.0) / 2.,
            Align::Right => bar_width - extent.0,
        });

        ctx.text(&self.text(), 1., self.text_style.padding)?;

        Ok(())
    }

    fn current_extent(&mut self, ctx: &mut dyn DrawContext) -> penrose::draw::Result<(f64, f64)> {
        let (l, r) = self.text_style.padding;

        ctx.font(&self.text_style.font, self.text_style.point_size)?;

        let (w, h) = ctx.text_extent(&self.text())?;
        let extent = (w + l + r + 0.1, h + 0.1);

        Ok(extent)
    }
}
