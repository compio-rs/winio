use std::ops::Deref;

use plotters::prelude::{Color as PlottersColor, *};
use winio::prelude::*;

use crate::{Error, Result};

pub struct PlottersPage {
    window: Child<TabViewItem>,
    label: Child<Label>,
    up_down: Child<UpDownEdit>,
    canvas: Child<Canvas>,
    n: usize,
}

#[derive(Debug)]
pub enum PlottersPageEvent {}

#[derive(Debug)]
pub enum PlottersPageMessage {
    ChangeExponent,
}

impl Component for PlottersPage {
    type Error = Error;
    type Event = PlottersPageEvent;
    type Init<'a> = &'a TabView;
    type Message = PlottersPageMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (init) => {
                text: "Plotters",
            },
            label: Label = (&window) => {
                text: "Exponent:",
                halign: HAlign::Right,
            },
            up_down: UpDownEdit = (&window) => {
                minimum: 1,
                maximum: 10,
                step: 1,
                value: 2,
            },
            canvas: Canvas = (&window),
        }

        let n = up_down.value()?;
        Ok(Self {
            window,
            label,
            up_down,
            canvas,
            n,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender,
            self.window => {},
            self.label => {},
            self.up_down => {
                UpDownEditEvent::Change => PlottersPageMessage::ChangeExponent,
            },
            self.canvas => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.label, self.up_down, self.canvas)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            PlottersPageMessage::ChangeExponent => {
                if let Ok(n) = self.up_down.value() {
                    self.n = n;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let size = self.window.size()?;
        {
            let mut grid = layout! {
                Grid::from_str("1*, 1*", "auto, 1*").unwrap(),
                self.label => { column: 0, row: 0, halign: HAlign::Right, valign: VAlign::Center },
                self.up_down => { column: 1, row: 0, halign: HAlign::Left, min_width: 60.0 },
                self.canvas => { column: 0, row: 1, column_span: 2, valign: VAlign::Stretch, halign: HAlign::Stretch },
            };
            grid.set_size(size)?;
        }

        let root = WinioCanvasBackend::new(&mut self.canvas)?.into_drawing_area();

        let fore = if ColorTheme::current()? == ColorTheme::Dark {
            WHITE
        } else {
            BLACK
        };
        const FONT: &str = "sans-serif";

        let title = format!("y=x^{}", self.n);
        let y_min = if self.n.is_multiple_of(2) {
            -0.1f32
        } else {
            -1.1f32
        };
        let mut chart = ChartBuilder::on(&root)
            .caption(title.as_str(), (FONT, 50).into_font().color(&fore))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, y_min..1.1f32)?;
        chart
            .configure_mesh()
            .axis_style(fore)
            .light_line_style(PlottersColor::mix(&fore, 0.1))
            .bold_line_style(PlottersColor::mix(&fore, 0.2))
            .label_style(FONT.into_font().color(&fore))
            .draw()?;
        chart
            .draw_series(LineSeries::new(
                (-100..=100).map(|x| {
                    let x = x as f32 / 100.0;
                    (x, x.powi(self.n as i32))
                }),
                &RED,
            ))?
            .label(title.as_str())
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
