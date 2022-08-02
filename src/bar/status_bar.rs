use penrose::{
    common::geometry::Region,
    core::Hook,
    draw::{Color, Draw, DrawContext, Position, TextStyle},
    xcb::XcbDraw,
    xconnection::{Atom, Prop, WinType, XConn},
    WindowManager, Xid,
};

use crate::{widgets, with_player, Align, BarWidget, Dracula, BAR_HEIGHT, FIRA};

pub type Sender = std::sync::mpsc::Sender<StatusBarEvent>;
pub type Receiver = std::sync::mpsc::Receiver<StatusBarEvent>;

pub enum StatusBarEvent {
    Startup,
    ScreensUpdated,
}

pub struct StatusBarHook {
    sender: Sender,
}

impl<X: XConn> Hook<X> for StatusBarHook {
    fn startup(&mut self, _wm: &mut WindowManager<X>) -> penrose::Result<()> {
        self.sender
            .send(StatusBarEvent::Startup)
            .expect("Failed to send event");

        Ok(())
    }

    fn screens_updated(
        &mut self,
        _wm: &mut WindowManager<X>,
        _dimensions: &[Region],
    ) -> penrose::Result<()> {
        self.sender
            .send(StatusBarEvent::ScreensUpdated)
            .expect("Failed to send event");

        Ok(())
    }
}

pub struct StatusBarWidgets {
    pub left: Vec<Box<dyn BarWidget>>,
    pub center: Option<Box<dyn BarWidget>>,
    pub right: Vec<Box<dyn BarWidget>>,
}

/// A simple status bar that works via hooks
pub struct StatusBar<D: Draw> {
    draw: D,
    position: Position,
    /// The widgets contained within this status bar
    pub widgets: StatusBarWidgets,
    /// (window ID, width)
    screens: Vec<(Xid, usize)>,
    height: usize,
    bg: Color,
    sender: Sender,
    receiver: Receiver,
}

// I don't know if this is safe or not
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<D: Draw> Send for StatusBar<D> {}
unsafe impl<D: Draw> Sync for StatusBar<D> {}

impl<D: Draw + 'static> StatusBar<D> {
    /// Try to initialise a new empty status bar.
    ///
    /// # Errors
    ///
    /// If we are unable to create our window, then we return an error
    pub fn try_new(
        drw: D,
        position: Position,
        height: usize,
        bg: impl Into<Color>,
        fonts: &[&str],
        widgets: StatusBarWidgets,
    ) -> penrose::Result<Self> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut bar = Self {
            draw: drw,
            position,
            widgets,
            screens: vec![],
            height,
            bg: bg.into(),
            sender,
            receiver,
        };
        bar.init_for_screens()?;
        fonts.iter().for_each(|f| bar.draw.register_font(f));

        Ok(bar)
    }

    fn init_for_screens(&mut self) -> penrose::Result<()> {
        for &(id, _) in &self.screens {
            self.draw.destroy_client(id)?;
        }

        let screen_sizes = self.draw.screen_sizes()?;

        self.screens = screen_sizes
            .into_iter()
            .map(|r| {
                let id = self.draw.new_window(
                    WinType::InputOutput(Atom::NetWindowTypeDock),
                    Region::new(
                        r.x,
                        match self.position {
                            Position::Top => r.y,
                            Position::Bottom => r.h - self.height as u32,
                        },
                        r.w,
                        self.height as u32,
                    ),
                    false,
                )?;

                let p = Prop::UTF8String(vec!["penrose-statusbar".to_string()]);
                for atom in &[Atom::NetWmName, Atom::WmName, Atom::WmClass] {
                    self.draw.change_prop(id, atom.as_ref(), p.clone())?;
                }

                self.draw.flush(id)?;
                Ok((id, r.w as usize))
            })
            .collect::<penrose::Result<Vec<(u32, usize)>>>()?;

        Ok(())
    }

    /// Re-render all widgets in this status bar
    ///
    /// # Errors
    ///
    /// If we are unable to re-render a widget, then we return an error
    pub fn redraw(&mut self) -> penrose::Result<()> {
        for &(window_id, width) in &self.screens {
            let mut ctx = self.draw.context_for(window_id)?;

            ctx.clear()?;

            ctx.color(&self.bg);
            ctx.rectangle(0.0, 0.0, width as f64, self.height as f64)?;

            for widget in &mut self.widgets.left {
                widget.draw(&mut ctx, Align::Left, width, self.height)?;
                ctx.flush();
            }

            if let Some(widget) = &mut self.widgets.center {
                widget.draw(&mut ctx, Align::Center, width, self.height)?;
                ctx.flush();
            }

            for widget in &mut self.widgets.right {
                widget.draw(&mut ctx, Align::Right, width, self.height)?;
                ctx.flush();
            }

            self.draw.flush(window_id)?;
        }

        Ok(())
    }

    pub fn create_hook(&self) -> StatusBarHook {
        StatusBarHook {
            sender: self.sender.clone(),
        }
    }

    /// Poll for events from the status bar
    ///
    /// # Errors
    ///
    /// Returns an error if a ran command fails,
    /// not if events can't be received.
    pub fn poll_events(&mut self) -> penrose::Result<()> {
        if let Ok(event) = self.receiver.try_recv() {
            match event {
                StatusBarEvent::ScreensUpdated => {
                    self.init_for_screens()?;
                    self.redraw()?;
                }
                StatusBarEvent::Startup => {
                    self.redraw()?;
                }
            }
        }

        Ok(())
    }

    pub fn spawn_thread(mut self) {
        std::thread::spawn(move || {
            let mut timer = std::time::Instant::now();

            loop {
                self.poll_events()
                    .expect("Penrose error while polling events");

                if timer.elapsed() > std::time::Duration::from_secs(1) {
                    timer = std::time::Instant::now();
                    self.redraw().expect("Failed to redraw bar");
                }
            }
        });
    }
}

impl Default for StatusBar<XcbDraw> {
    fn default() -> Self {
        let draw = XcbDraw::new().expect("Failed to create XCB draw");

        let text_style = TextStyle {
            font: FIRA.to_string(),
            point_size: 12,
            fg: Dracula::FG.into(),
            bg: None,
            padding: (3., 3.),
        };

        Self::try_new(
            draw,
            penrose::draw::Position::Top,
            BAR_HEIGHT,
            Dracula::BG,
            &[FIRA],
            StatusBarWidgets {
                left: vec![widgets::Text::new(
                    || {
                        use chrono::prelude::*;

                        let now = Local::now();

                        Some(now.format("%e %h %Y - %k:%M:%S").to_string())
                    },
                    text_style.clone(),
                )],
                center: Some(widgets::Text::new(
                    || {
                        with_player(|player| {
                            let metadata = player.get_metadata().ok()?;

                            let title = match metadata.title() {
                                Some(title) if title.trim().is_empty() => None,
                                Some(title) => Some(title.to_string()),
                                None => None,
                            };

                            let artists = metadata.artists().and_then(|artists| {
                                let artists = artists
                                    .iter()
                                    .filter(|a| !a.trim().is_empty())
                                    .map(|a| a as &str)
                                    .collect::<Vec<_>>();

                                if artists.is_empty() {
                                    None
                                } else {
                                    Some(artists.join(", "))
                                }
                            });

                            match (title, artists) {
                                (Some(title), Some(artists)) => {
                                    Some(format!("{} - {}", title, artists))
                                }
                                (Some(title), None) => Some(title),
                                (None, Some(artists)) => Some(artists),
                                _ => None,
                            }
                        })
                    },
                    text_style.clone(),
                )),
                right: vec![widgets::Text::new(|| Some(" ".to_string()), text_style)],
            },
        )
        .expect("Failed to create status bar")
    }
}
