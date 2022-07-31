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
    align: Align,
    extent: Option<(f64, f64)>,
    last_updated: std::time::Instant,
}

impl ReactiveText {
    pub fn new(
        text: impl FnMut() -> Option<String> + 'static,
        text_style: TextStyle,
        align: Align,
    ) -> Box<Self> {
        Box::new(Self {
            text: Box::new(text),
            text_style,
            align,
            extent: None,
            last_updated: std::time::Instant::now(),
        })
    }

    pub fn text(&mut self) -> String {
        (self.text)().unwrap_or_else(|| "".to_string())
    }

    fn calc_extent(
        &mut self,
        ctx: &mut dyn DrawContext,
    ) -> Result<(f64, f64), penrose::draw::Error> {
        let (l, r) = self.text_style.padding;

        ctx.font(&self.text_style.font, self.text_style.point_size)?;

        let (w, h) = ctx.text_extent(&self.text())?;
        let extent = (w + l + r + 0.1, h + 0.1);

        Ok(extent)
    }
}

impl BarWidget for ReactiveText {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        bar_width: f64,
        _bar_height: f64,
    ) -> Result<(), penrose::draw::Error> {
        // Update the text.
        let text = self.text();

        // Recalculate the extent with the new text.
        self.extent = Some(self.calc_extent(ctx)?);
        let extent = self.extent.unwrap();

        ctx.font(&self.text_style.font, self.text_style.point_size)?;
        ctx.color(&self.text_style.fg);

        ctx.set_x_offset(match self.align {
            Align::Left => 0.,
            Align::Center => (bar_width - extent.0) / 2.,
            Align::Right => bar_width - extent.0,
        });

        ctx.text(&text, 1., self.text_style.padding)?;

        self.last_updated = std::time::Instant::now();

        Ok(())
    }

    fn current_extent(
        &mut self,
        ctx: &mut dyn DrawContext,
    ) -> Result<(f64, f64), penrose::draw::Error> {
        match self.extent {
            Some(extent) => Ok(extent),
            None => {
                let extent = self.calc_extent(ctx)?;
                self.extent = Some(extent);
                Ok(extent)
            }
        }
    }
}
