use crate::emath::{Pos2, Rect, Vec2};

/// An input event from the backend into egui, about a specific [viewport](crate::viewport).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ViewportEvent {
    /// The user clicked the close-button on the window, or similar.
    ///
    /// If this is the root viewport, the application will exit
    /// after this frame unless you send a
    /// [`crate::ViewportCommand::CancelClose`] command.
    ///
    /// If this is not the root viewport,
    /// it is up to the user to hide this viewport the next frame.
    ///
    /// This even will wake up both the child and parent viewport.
    Close,
}

/// A monitor (display) on the desktop, as reported by the backend.
///
/// Positions and sizes are desktop coordinates in each monitor's own logical points:
/// the native physical pixels divided by [`Self::scale_factor`] of that monitor.
/// The primary monitor usually has its top-left corner at the origin,
/// so monitors to the left of it or above it have negative coordinates.
/// When monitors have different scale factors, their logical rectangles need not touch or line up,
/// because each is scaled by its own factor.
///
/// On Wayland, [`Self::position`] is the compositor's layout position when it reports one
/// and `(0, 0)` otherwise, and window positions are unknown
/// ([`ViewportInfo::outer_rect`] is `None`), so a window can't be placed on a monitor by position.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MonitorInfo {
    /// Human-readable name of the monitor, if known.
    pub name: Option<String>,

    /// The top-left corner of the monitor on the desktop, in the monitor's logical points.
    ///
    /// May be negative. On Wayland, `(0, 0)` unless the compositor reports a layout position.
    pub position: Pos2,

    /// The size of the monitor, in its logical points.
    pub size: Vec2,

    /// Native physical pixels per logical point on this monitor.
    pub scale_factor: f32,
}

impl MonitorInfo {
    /// The monitor's area on the desktop: [`Self::position`] and [`Self::size`].
    pub fn rect(&self) -> Rect {
        Rect::from_min_size(self.position, self.size)
    }
}

/// Information about the current viewport, given as input each frame.
///
/// `None` means "unknown".
///
/// All units are in ui "points", which can be calculated from native physical pixels
/// using `pixels_per_point` = [`crate::Context::zoom_factor`] * `[Self::native_pixels_per_point`];
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ViewportInfo {
    /// Parent viewport, if known.
    pub parent: Option<crate::ViewportId>,

    /// Name of the viewport, if known.
    pub title: Option<String>,

    pub events: Vec<ViewportEvent>,

    /// The OS native pixels-per-point.
    ///
    /// This should always be set, if known.
    ///
    /// On web this takes browser scaling into account,
    /// and corresponds to [`window.devicePixelRatio`](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio) in JavaScript.
    pub native_pixels_per_point: Option<f32>,

    /// Current monitor size in egui points.
    pub monitor_size: Option<Vec2>,

    /// The monitor the viewport is mostly on, if known.
    ///
    /// See [`MonitorInfo`] for its coordinate space. Unlike [`Self::monitor_size`],
    /// it ignores [`crate::Context::zoom_factor`].
    pub monitor: Option<MonitorInfo>,

    /// All monitors on the desktop, if known; empty if unknown.
    ///
    /// See [`MonitorInfo`] for their coordinate space.
    ///
    /// `egui-winit` refreshes this list when the viewport is created and when its window is moved,
    /// resized or changes scale factor, at most once a second.
    pub monitors: Vec<MonitorInfo>,

    /// The inner rectangle of the native window, in monitor space and ui points scale.
    ///
    /// This is the content rectangle of the viewport.
    ///
    /// **`eframe` notes**:
    ///
    /// On Android / Wayland, this will always be `None` since getting the
    /// position of the window is not possible.
    pub inner_rect: Option<Rect>,

    /// The outer rectangle of the native window, in monitor space and ui points scale.
    ///
    /// This is the content rectangle plus decoration chrome.
    ///
    /// **`eframe` notes**:
    ///
    /// On Android / Wayland, this will always be `None` since getting the
    /// position of the window is not possible.
    pub outer_rect: Option<Rect>,

    /// Are we minimized?
    pub minimized: Option<bool>,

    /// Are we maximized?
    pub maximized: Option<bool>,

    /// Are we in fullscreen mode?
    pub fullscreen: Option<bool>,

    /// Is the window focused and able to receive input?
    ///
    /// This should be the same as [`RawInput::focused`](crate::RawInput::focused).
    pub focused: Option<bool>,

    /// Is the window fully occluded (completely covered) by another window?
    ///
    /// Not all platforms support this.
    /// On platforms that don't, this will be `None` or `Some(false)`.
    pub occluded: Option<bool>,
}

impl ViewportInfo {
    /// Is the window considered visible for rendering purposes?
    ///
    /// A window is not visible if it is minimized or occluded.
    /// When not visible, the UI is not painted and rendering is skipped,
    /// but application logic may still be executed by some integrations.
    ///
    /// Send [`crate::ViewportCommand::RequestPaintWhileHidden`] to have the ui run and the
    /// frame painted anyway, e.g. to screenshot or drive an app that sits in the background.
    pub fn visible(&self) -> Option<bool> {
        match (self.minimized, self.occluded) {
            (Some(true), _) | (_, Some(true)) => Some(false),
            (Some(false), Some(false)) => Some(true),
            (_, None) | (None, _) => None,
        }
    }

    /// This viewport has been told to close.
    ///
    /// If this is the root viewport, the application will exit
    /// after this frame unless you send a
    /// [`crate::ViewportCommand::CancelClose`] command.
    ///
    /// If this is not the root viewport,
    /// it is up to the user to hide this viewport the next frame.
    pub fn close_requested(&self) -> bool {
        self.events.contains(&ViewportEvent::Close)
    }

    /// Helper: move [`Self::events`], clone the other fields.
    pub fn take(&mut self) -> Self {
        Self {
            parent: self.parent,
            title: self.title.clone(),
            events: core::mem::take(&mut self.events),
            native_pixels_per_point: self.native_pixels_per_point,
            monitor_size: self.monitor_size,
            monitor: self.monitor.clone(),
            monitors: self.monitors.clone(),
            inner_rect: self.inner_rect,
            outer_rect: self.outer_rect,
            minimized: self.minimized,
            maximized: self.maximized,
            fullscreen: self.fullscreen,
            focused: self.focused,
            occluded: self.occluded,
        }
    }

    pub fn ui(&self, ui: &mut crate::Ui) {
        let Self {
            parent,
            title,
            events,
            native_pixels_per_point,
            monitor_size,
            monitor,
            monitors,
            inner_rect,
            outer_rect,
            minimized,
            maximized,
            fullscreen,
            focused,
            occluded,
        } = self;

        crate::Grid::new("viewport_info").show(ui, |ui| {
            ui.label("Parent:");
            ui.label(opt_as_str(parent));
            ui.end_row();

            ui.label("Title:");
            ui.label(opt_as_str(title));
            ui.end_row();

            ui.label("Events:");
            ui.label(format!("{events:?}"));
            ui.end_row();

            ui.label("Native pixels-per-point:");
            ui.label(opt_as_str(native_pixels_per_point));
            ui.end_row();

            ui.label("Monitor size:");
            ui.label(opt_as_str(monitor_size));
            ui.end_row();

            ui.label("Monitor:");
            ui.label(monitor.as_ref().map_or(String::new(), monitor_as_string));
            ui.end_row();

            ui.label("Monitors:");
            ui.label(
                monitors
                    .iter()
                    .map(monitor_as_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            ui.end_row();

            ui.label("Inner rect:");
            ui.label(opt_rect_as_string(inner_rect));
            ui.end_row();

            ui.label("Outer rect:");
            ui.label(opt_rect_as_string(outer_rect));
            ui.end_row();

            ui.label("Minimized:");
            ui.label(opt_as_str(minimized));
            ui.end_row();

            ui.label("Maximized:");
            ui.label(opt_as_str(maximized));
            ui.end_row();

            ui.label("Fullscreen:");
            ui.label(opt_as_str(fullscreen));
            ui.end_row();

            ui.label("Focused:");
            ui.label(opt_as_str(focused));
            ui.end_row();

            ui.label("Occluded:");
            ui.label(opt_as_str(occluded));
            ui.end_row();

            let visible = self.visible();

            ui.label("Visible:");
            ui.label(opt_as_str(&visible));
            ui.end_row();

            #[expect(clippy::ref_option)]
            fn opt_rect_as_string(v: &Option<Rect>) -> String {
                v.as_ref().map_or(String::new(), |r| {
                    format!("Pos: {:?}, size: {:?}", r.min, r.size())
                })
            }

            fn monitor_as_string(m: &MonitorInfo) -> String {
                format!(
                    "{}: pos: {:?}, size: {:?}, scale: {}",
                    m.name.as_deref().unwrap_or("?"),
                    m.position,
                    m.size,
                    m.scale_factor
                )
            }

            #[expect(clippy::ref_option)]
            fn opt_as_str<T: core::fmt::Debug>(v: &Option<T>) -> String {
                v.as_ref().map_or(String::new(), |v| format!("{v:?}"))
            }
        });
    }
}
