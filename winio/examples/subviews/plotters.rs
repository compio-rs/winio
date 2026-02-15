use std::ops::Deref;

use plotters::prelude::{Color as _, *};
use plotters_backend::DrawingErrorKind;
use winio::prelude::*;

use crate::{Error, Result};

pub struct PlottersPage {
    window: Child<TabViewItem>,
    canvas: Child<Canvas>,
}

#[derive(Debug)]
pub enum PlottersPageEvent {}

#[derive(Debug)]
pub enum PlottersPageMessage {}

impl Component for PlottersPage {
    type Error = Error;
    type Event = PlottersPageEvent;
    type Init<'a> = ();
    type Message = PlottersPageMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: "Plotters",
            },
            canvas: Canvas = (&window),
        }

        Ok(Self { window, canvas })
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let size = self.window.size()?;
        self.canvas.set_rect(size.into())?;

        let root = WinioCanvasBackend::new(&mut self.canvas)?.into_drawing_area();

        let fore = if ColorTheme::current()? == ColorTheme::Dark {
            WHITE
        } else {
            BLACK
        };
        const FONT: &str = "sans-serif";

        let mut chart = ChartBuilder::on(&root)
            .caption("y=x^2", (FONT, 50).into_font().color(&fore))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;
        chart
            .configure_mesh()
            .axis_style(fore)
            .light_line_style(fore.mix(0.1))
            .bold_line_style(fore.mix(0.2))
            .label_style(FONT.into_font().color(&fore))
            .draw()?;
        chart
            .draw_series(LineSeries::new(
                (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                &RED,
            ))?
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .background_style(TRANSPARENT)
            .border_style(fore)
            .label_font(FONT.into_font().color(&fore))
            .draw()?;
        root.present()?;

        Ok(())
    }
}

impl Deref for PlottersPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl From<DrawingAreaErrorKind<winio::Error>> for Error {
    fn from(value: DrawingAreaErrorKind<winio::Error>) -> Self {
        match value {
            DrawingAreaErrorKind::BackendError(DrawingErrorKind::DrawingError(e)) => e.into(),
            DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(e)) => {
                Error::Io(std::io::Error::other(e))
            }
            _ => Error::Io(std::io::Error::other(value)),
        }
    }
}
