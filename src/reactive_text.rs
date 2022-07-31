use penrose::{
    common::geometry::Region,
    draw::{DrawContext, TextStyle, Widget},
};

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
    screen_dimensions: Option<Vec<Region>>,
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
            screen_dimensions: None,
            last_updated: std::time::Instant::now(),
        })
    }

    pub fn text(&mut self) -> String {
        (self.text)().unwrap_or_else(|| "".to_string())
    }

    fn calc_extent(
        &mut self,
        ctx: &mut dyn DrawContext,
        _h: f64,
    ) -> Result<(f64, f64), penrose::draw::Error> {
        let (l, r) = self.text_style.padding;

        ctx.font(&self.text_style.font, self.text_style.point_size)?;

        let (w, h) = ctx.text_extent(&self.text())?;
        let extent = (w + l + r + 0.1, h + 0.1);

        Ok(extent)
    }
}

impl<X> penrose::core::Hook<X> for ReactiveText
where
    X: penrose::xconnection::XConn,
{
    fn screens_updated(
        &mut self,
        _wm: &mut penrose::WindowManager<X>,
        dimensions: &[Region],
    ) -> penrose::Result<()> {
        self.screen_dimensions = Some(dimensions.to_vec());
        Ok(())
    }
}

impl Widget for ReactiveText {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        _screen: usize,
        _screen_has_focus: bool,
        w: f64,
        h: f64,
    ) -> Result<(), penrose::draw::Error> {
        // Update the text.
        let text = self.text();

        // Recalculate the extent with the new text.
        self.extent = Some(self.calc_extent(ctx, h)?);
        let extent = self.extent.unwrap();

        ctx.font(&self.text_style.font, self.text_style.point_size)?;
        ctx.color(&self.text_style.fg);

        ctx.set_x_offset(match self.align {
            Align::Left => 0.,
            Align::Center => (w - extent.0) / 2.,
            Align::Right => w - extent.0,
        });

        ctx.text(&text, 1., self.text_style.padding)?;

        self.last_updated = std::time::Instant::now();

        Ok(())
    }

    fn current_extent(
        &mut self,
        ctx: &mut dyn DrawContext,
        h: f64,
    ) -> Result<(f64, f64), penrose::draw::Error> {
        match self.extent {
            Some(extent) => Ok(extent),
            None => {
                let extent = self.calc_extent(ctx, h)?;
                self.extent = Some(extent);
                Ok(extent)
            }
        }
    }

    fn require_draw(&self) -> bool {
        panic!("This function shouldn't be called");
    }

    fn is_greedy(&self) -> bool {
        panic!("This function shouldn't be called");
    }
}
