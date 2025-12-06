use std::{ops::Deref, path::PathBuf, time::Duration};

use compio::{runtime::spawn, time::interval};
use url::Url;
use winio::prelude::*;

use crate::{Error, Result};

pub struct MediaPage {
    window: Child<TabViewItem>,
    media: Child<Media>,
    playing: bool,
    play_button: Child<Button>,
    browse_button: Child<Button>,
    time_slider: Child<Slider>,
    time_label: Child<Label>,
    volume_slider: Child<Slider>,
    volume_label: Child<Label>,
}

impl MediaPage {
    fn set_playing(&mut self, v: bool) -> Result<()> {
        self.playing = v;
        self.play_button
            .set_text(if self.playing { "⏸️" } else { "▶️" })?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum MediaPageEvent {
    ShowMessage(MessageBox),
    ChooseFile,
}

#[derive(Debug)]
pub enum MediaPageMessage {
    Noop,
    Tick,
    Volume,
    Time,
    Play,
    ChooseFile,
    OpenFile(PathBuf),
}

impl Component for MediaPage {
    type Error = Error;
    type Event = MediaPageEvent;
    type Init<'a> = ();
    type Message = MediaPageMessage;

    async fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: "Media",
            },
            media: Media = (&window),
            play_button: Button = (&window) => {
                enabled: false,
                text: "▶️"
            },
            browse_button: Button = (&window) => {
                text: "..."
            },
            time_slider: Slider = (&window) => {
                enabled: false,
                tick_pos: TickPosition::TopLeft,
                minimum: 0,
            },
            time_label: Label = (&window) => {
                halign: HAlign::Right,
            },
            volume_slider: Slider = (&window) => {
                enabled: false,
                tick_pos: TickPosition::TopLeft,
                minimum: 0,
                maximum: 100,
                pos: 100,
                freq: 20,
            },
            volume_label: Label = (&window),
        }
        sender.post(MediaPageMessage::Volume);

        let sender = sender.clone();
        spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                sender.post(MediaPageMessage::Tick);
            }
        })
        .detach();

        Ok(Self {
            window,
            media,
            playing: false,
            play_button,
            browse_button,
            time_slider,
            time_label,
            volume_slider,
            volume_label,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MediaPageMessage::Noop,
            self.volume_slider => {
                SliderEvent::Change => MediaPageMessage::Volume,
            },
            self.time_slider => {
                SliderEvent::Change => MediaPageMessage::Time,
            },
            self.play_button => {
                ButtonEvent::Click => MediaPageMessage::Play,
            },
            self.browse_button => {
                ButtonEvent::Click => MediaPageMessage::ChooseFile,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.media,
            self.play_button,
            self.browse_button,
            self.time_slider,
            self.volume_slider,
            self.volume_label
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MediaPageMessage::Noop => Ok(false),
            MediaPageMessage::Tick => {
                fn format_duration(dur: Duration) -> String {
                    let secs = dur.as_secs();
                    let hours = secs / 3600;
                    let minutes = (secs % 3600) / 60;
                    let seconds = secs % 60;
                    if hours > 0 {
                        format!("{hours:02}:{minutes:02}:{seconds:02}")
                    } else {
                        format!("{minutes:02}:{seconds:02}")
                    }
                }

                let ct = self.media.current_time()?;
                let ft = self.media.full_time()?;
                if let Some(ft) = ft {
                    let ft_secs = ft.as_secs_f64();
                    self.time_slider.set_freq((ft_secs * 100.0) as usize / 10)?;
                    self.time_slider.set_maximum((ft_secs * 100.0) as _)?;
                    self.time_slider.set_pos((ct.as_secs_f64() * 100.0) as _)?;
                    self.time_label.set_text(format!(
                        "{} / {}",
                        format_duration(ct),
                        format_duration(ft)
                    ))?;
                } else {
                    self.time_slider.set_maximum(1)?;
                    self.time_slider.set_pos(0)?;
                    self.time_slider.set_freq(1)?;
                    self.time_label.set_text(format_duration(ct))?;
                }
                Ok(true)
            }
            MediaPageMessage::Volume => {
                let pos = self.volume_slider.pos()?;
                self.volume_label.set_text(pos.to_string())?;
                self.media.set_volume(pos as f64 / 100.0)?;
                Ok(true)
            }
            MediaPageMessage::Time => {
                let pos = self.time_slider.pos()?;
                let ft = self.media.full_time()?;
                if ft.is_some() {
                    self.media
                        .set_current_time(Duration::from_secs_f64(pos as f64 / 100.0))?;
                }
                Ok(true)
            }
            MediaPageMessage::Play => {
                if self.playing {
                    self.media.pause()?;
                    self.set_playing(false)?;
                } else {
                    self.media.play()?;
                    self.set_playing(true)?;
                }
                Ok(true)
            }
            MediaPageMessage::ChooseFile => {
                sender.output(MediaPageEvent::ChooseFile);
                Ok(false)
            }
            MediaPageMessage::OpenFile(p) => {
                let url =
                    Url::from_file_path(&p).map_err(|_| std::io::ErrorKind::InvalidFilename)?;
                match self.media.load(url.as_str()).await {
                    Ok(()) => {
                        self.volume_slider.enable()?;
                        self.time_slider.enable()?;
                        self.play_button.enable()?;
                        self.media.play()?;
                        self.set_playing(true)?;
                    }
                    Err(e) => {
                        self.volume_slider.disable()?;
                        self.time_slider.disable()?;
                        self.play_button.disable()?;
                        self.set_playing(false)?;
                        sender.output(MediaPageEvent::ShowMessage(
                            MessageBox::new()
                                .buttons(MessageBoxButton::Ok)
                                .style(MessageBoxStyle::Error)
                                .message(format!("Failed to load media file: {}", e)),
                        ));
                    }
                }
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;
        {
            let margin = Margin::new_all_same(4.0);

            let mut bottom_bar = layout! {
                StackPanel::new(Orient::Horizontal),
                self.play_button   => { margin: margin },
                self.time_slider   => { margin: margin, grow: true },
                self.time_label    => { margin: margin, valign: VAlign::Center, halign: HAlign::Center },
                self.volume_slider => { margin: margin, width: 200.0 },
                self.volume_label  => { margin: margin, valign: VAlign::Center, halign: HAlign::Left, width: 20.0 },
                self.browse_button => { margin: margin }
            };
            let mut grid = layout! {
                Grid::from_str("1*", "1*,auto").unwrap(),
                self.media => { column: 0, row: 0 },
                bottom_bar => { column: 0, row: 1 },
            };
            grid.set_size(csize)?;
        }
        Ok(())
    }
}

impl Deref for MediaPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
