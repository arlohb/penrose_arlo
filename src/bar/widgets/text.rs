use std::sync::Arc;

use penrose::draw::DrawContext;

use crate::{Align, BarWidget, WidgetStyle};

pub struct Text {
    text: Box<dyn FnMut() -> Option<String>>,
    widget_style: Arc<WidgetStyle>,
}

impl Text {
    pub fn new(
        text: impl FnMut() -> Option<String> + 'static,
        widget_style: Arc<WidgetStyle>,
    ) -> Box<Self> {
        Box::new(Self {
            text: Box::new(text),
            widget_style,
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
        font: &str,
        align: Align,
        offset: usize,
        bar_width: usize,
        _bar_height: usize,
    ) -> penrose::draw::Result<usize> {
        let current_width = self.current_width(ctx, font)?;

        ctx.font(font, self.widget_style.text_size as i32)?;
        ctx.color(&self.widget_style.fg);

        ctx.set_x_offset(match align {
            Align::Left => offset,
            Align::Center => (bar_width - current_width) / 2 + offset,
            Align::Right => bar_width - current_width - offset,
        } as f64);

        ctx.set_y_offset(self.widget_style.y_offset as f64);

        ctx.text(&self.text(), 1., (0., 0.))?;

        Ok(current_width + self.widget_style.x_padding)
    }

    fn current_width(
        &mut self,
        ctx: &mut dyn DrawContext,
        font: &str,
    ) -> penrose::draw::Result<usize> {
        ctx.font(font, self.widget_style.text_size as i32)?;

        let width = ctx.text_extent(&self.text())?.0 as usize;

        Ok(width + self.widget_style.x_padding)
    }
}
