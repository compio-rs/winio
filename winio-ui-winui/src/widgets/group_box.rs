use inherit_methods_macro::inherit_methods;
use windows_core::{HSTRING, Interface, h};
use windows_reference::IReference;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{
    self as MUX, Controls as MUXC, Media as MUXM, Shapes as MUXS, TextWrapping, Visibility,
};

use crate::{Result, View, Widget, widgets::Convertible};

const STROKE: f64 = 1.0;
const RADIUS: f64 = 8.0;
const HEADING_MARGIN: f64 = 16.0;
const BORDER_END_PADDING: f64 = 3.0;
const BORDER_START_PADDING: f64 = 4.0;
const HEADING_BASELINE: f64 = 0.61;

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
    path: MUXS::Path,
    heading: MUXC::TextBlock,
    view: View,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let root = MUXC::Canvas::new()?;

        let path = MUXS::Path::new()?;
        path.SetStrokeThickness(STROKE)?;
        path.SetStretch(MUXM::Stretch::None)?;
        path.SetIsHitTestVisible(false)?;
        path.SetOpacity(0.5)?;
        path.SetVisibility(Visibility::Collapsed)?;

        let heading = MUXC::TextBlock::new()?;
        heading.SetTextWrapping(TextWrapping::NoWrap)?;

        root.Children()?.Append(&path)?;
        root.Children()?.Append(&heading)?;

        let view = View::new(BorrowedContainer::winui(&root))?;

        {
            let path = path.clone();
            root.ActualThemeChanged(move |_, _| {
                if let Ok(brush) = border_brush() {
                    let _ = path.SetStroke(&brush);
                }
                Ok(())
            })?
            .forget();
        }

        Ok(Self {
            handle: Widget::new(parent, root.cast()?)?,
            path,
            heading,
            view,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.update_layout()
    }

    pub fn preferred_size(&self) -> Result<Size> {
        self.measure_heading()
    }

    pub fn client_size(&self) -> Result<Size> {
        self.view.size()
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.heading.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.heading.SetText(&HSTRING::from(s.as_ref()))?;
        self.update_layout()
    }

    fn layout_size(&self) -> Result<Size> {
        self.handle.size()
    }

    fn measure_heading(&self) -> Result<Size> {
        self.heading
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())?;
        Ok(Size::from_native(self.heading.DesiredSize()?))
    }

    fn update_layout(&mut self) -> Result<()> {
        let size = self.layout_size()?;
        let heading = self.measure_heading()?;

        MUXC::Canvas::SetLeft(&self.heading, HEADING_MARGIN)?;
        MUXC::Canvas::SetTop(&self.heading, 0.0)?;

        let content_top = heading.height.max(STROKE);
        let content_width = (size.width - RADIUS * 2.0).max(0.0);
        let content_height = (size.height - content_top - RADIUS).max(0.0);
        self.view.set_loc(Point::new(RADIUS, content_top))?;
        self.view
            .set_size(Size::new(content_width, content_height))?;

        MUXC::Canvas::SetLeft(&self.path, 0.0)?;
        MUXC::Canvas::SetTop(&self.path, 0.0)?;
        self.path.SetWidth(size.width)?;
        self.path.SetHeight(size.height)?;

        if size.width > STROKE && size.height > STROKE {
            self.path.SetStroke(&border_brush()?)?;
            self.path.SetData(&build_border_geometry(size, heading)?)?;
            self.path.SetVisibility(Visibility::Visible)?;
        } else {
            self.path.SetVisibility(Visibility::Collapsed)?;
        }

        Ok(())
    }
}

fn build_border_geometry(size: Size, heading: Size) -> Result<MUXM::PathGeometry> {
    let width = size.width;
    let height = size.height;
    let half = STROKE / 2.0;

    let max_radius = ((width.min(height) - STROKE) / 2.0).max(0.0);
    let radius = RADIUS.min(max_radius);
    let top_start = radius + half;
    let top_end = width - radius - half;

    let has_heading = heading.width > 0.0 && heading.height > 0.0;
    let line_y = if has_heading {
        (heading.height * HEADING_BASELINE)
            .clamp(top_start, (height - radius - half).max(top_start))
    } else {
        half
    };

    let figure = MUXM::PathFigure::new()?;
    figure.SetIsFilled(false)?;
    let segments = figure.Segments()?;

    if has_heading {
        let gap_start = (HEADING_MARGIN - BORDER_END_PADDING).clamp(top_start, top_end);
        let gap_end =
            (HEADING_MARGIN + heading.width + BORDER_START_PADDING).clamp(top_start, top_end);

        figure.SetIsClosed(false)?;
        let start_x = if gap_end < top_end { gap_end } else { top_end };
        figure.SetStartPoint(Point::new(start_x, line_y).to_native())?;
        if gap_end < top_end {
            push_line(&segments, top_end, line_y)?;
        }
        push_border_after_top(&segments, width, height, line_y, radius, half)?;
        if top_start < gap_start {
            push_line(&segments, gap_start, line_y)?;
        }
    } else {
        figure.SetIsClosed(true)?;
        figure.SetStartPoint(Point::new(top_start, line_y).to_native())?;
        push_line(&segments, top_end, line_y)?;
        push_border_after_top(&segments, width, height, line_y, radius, half)?;
    }

    let geometry = MUXM::PathGeometry::new()?;
    geometry.Figures()?.Append(&figure)?;
    Ok(geometry)
}

fn push_border_after_top(
    segments: &MUXM::PathSegmentCollection,
    width: f64,
    height: f64,
    line_y: f64,
    radius: f64,
    half: f64,
) -> Result<()> {
    push_arc(segments, width - half, line_y + radius, radius)?;
    push_line(segments, width - half, height - radius - half)?;
    push_arc(segments, width - radius - half, height - half, radius)?;
    push_line(segments, radius + half, height - half)?;
    push_arc(segments, half, height - radius - half, radius)?;
    push_line(segments, half, line_y + radius)?;
    push_arc(segments, radius + half, line_y, radius)?;
    Ok(())
}

fn push_line(segments: &MUXM::PathSegmentCollection, x: f64, y: f64) -> Result<()> {
    let line = MUXM::LineSegment::new()?;
    line.SetPoint(Point::new(x, y).to_native())?;
    segments.Append(&line)?;
    Ok(())
}

fn push_arc(segments: &MUXM::PathSegmentCollection, x: f64, y: f64, radius: f64) -> Result<()> {
    if radius <= 0.0 {
        return push_line(segments, x, y);
    }
    let arc = MUXM::ArcSegment::new()?;
    arc.SetPoint(Point::new(x, y).to_native())?;
    arc.SetSize(Size::new(radius, radius).to_native())?;
    arc.SetRotationAngle(90.0)?;
    arc.SetIsLargeArc(false)?;
    arc.SetSweepDirection(MUXM::SweepDirection::Clockwise)?;
    segments.Append(&arc)?;
    Ok(())
}

fn border_brush() -> Result<MUXM::Brush> {
    lookup_brush(h!("ControlStrongStrokeColorDefaultBrush"))
        .or_else(|_| lookup_brush(h!("ControlStrokeColorDefaultBrush")))
}

fn lookup_brush(key: &HSTRING) -> Result<MUXM::Brush> {
    let resources = MUX::Application::Current()?.Resources()?;
    let key = IReference::<HSTRING>::from(key);
    resources.Lookup(&key)?.cast()
}

winio_handle::impl_as_widget!(GroupBox, handle);
winio_handle::impl_as_container!(GroupBox, view);
