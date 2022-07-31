use penrose::draw::{DrawContext, TextStyle};

use crate::{Align, BarWidget};

pub struct Text {
    text: Box<dyn FnMut() -> Option<String>>,
    text_style: TextStyle,
}

impl Text {
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

impl BarWidget for Text {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        align: Align,
        bar_width: usize,
        _bar_height: usize,
    ) -> penrose::draw::Result<()> {
        let current_width = self.current_width(ctx)?;

        ctx.font(&self.text_style.font, self.text_style.point_size)?;
        ctx.color(&self.text_style.fg);

        ctx.set_x_offset(match align {
            Align::Left => 0,
            Align::Center => (bar_width - current_width) / 2,
            Align::Right => bar_width - current_width,
        } as f64);

        ctx.text(&self.text(), 1., self.text_style.padding)?;

        Ok(())
    }

    fn current_width(&mut self, ctx: &mut dyn DrawContext) -> penrose::draw::Result<usize> {
        let (l, r) = (
            self.text_style.padding.0 as usize,
            self.text_style.padding.1 as usize,
        );

        ctx.font(&self.text_style.font, self.text_style.point_size)?;

        let unpadded = ctx.text_extent(&self.text())?.0 as usize;
        let padded = unpadded + l + r;

        Ok(padded)
    }
}
