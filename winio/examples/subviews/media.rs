use std::{ops::Deref, path::PathBuf, time::Duration};

use compio::{runtime::spawn, time::interval};
use tuplex::IntoArray;
use url::Url;
use winio::prelude::*;

pub struct MediaPage {
    window: Child<TabViewItem>,
    media: Child<Media>,
    playing: bool,
    play_button: Child<Button>,
    browse_button: Child<Button>,
    time_slider: Child<Slider>,
    volume_slider: Child<Slider>,
    volume_label: Child<Label>,
}

impl MediaPage {
    fn set_playing(&mut self, v: bool) {
        self.playing = v;
        self.play_button
            .set_text(if self.playing { "⏸️" } else { "▶️" });
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
    type Event = MediaPageEvent;
    type Init<'a> = &'a TabView;
    type Message = MediaPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: TabViewItem = (tabview) => {
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

        Self {
            window,
            media,
            playing: false,
            play_button,
            browse_button,
            time_slider,
            volume_slider,
            volume_label,
        }
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

    async fn update_children(&mut self) -> bool {
        futures_util::join!(
            self.window.update(),
            self.media.update(),
            self.play_button.update(),
            self.browse_button.update(),
            self.time_slider.update(),
            self.volume_slider.update(),
            self.volume_label.update(),
        )
        .into_array()
        .into_iter()
        .any(|x| x)
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MediaPageMessage::Noop => false,
            MediaPageMessage::Tick => {
                let ct = self.media.current_time();
                let ft = self.media.full_time();
                if let Some(ft) = ft {
                    let ft = ft.as_secs_f64();
                    self.time_slider.set_freq((ft * 100.0) as usize / 10);
                    self.time_slider.set_maximum((ft * 100.0) as _);
                    self.time_slider.set_pos((ct.as_secs_f64() * 100.0) as _);
                } else {
                    self.time_slider.set_maximum(1);
                    self.time_slider.set_pos(0);
                    self.time_slider.set_freq(1);
                }
                true
            }
            MediaPageMessage::Volume => {
                let pos = self.volume_slider.pos();
                self.volume_label.set_text(pos.to_string());
                self.media.set_volume(pos as f64 / 100.0);
                true
            }
            MediaPageMessage::Time => {
                let pos = self.time_slider.pos();
                let ft = self.media.full_time();
                if ft.is_some() {
                    self.media
                        .set_current_time(Duration::from_secs_f64(pos as f64 / 100.0));
                }
                true
            }
            MediaPageMessage::Play => {
                if self.playing {
                    self.media.pause();
                    self.set_playing(false);
                } else {
                    self.media.play();
                    self.set_playing(true);
                }
                true
            }
            MediaPageMessage::ChooseFile => {
                sender.output(MediaPageEvent::ChooseFile);
                false
            }
            MediaPageMessage::OpenFile(p) => {
                let url = Url::from_file_path(&p).unwrap();
                if self.media.load(url.as_str()).await {
                    self.volume_slider.enable();
                    self.time_slider.enable();
                    self.play_button.enable();
                    self.media.play();
                    self.set_playing(true);
                } else {
                    self.volume_slider.disable();
                    self.time_slider.disable();
                    self.play_button.disable();
                    self.set_playing(false);
                    sender.output(MediaPageEvent::ShowMessage(
                        MessageBox::new()
                            .buttons(MessageBoxButton::Ok)
                            .style(MessageBoxStyle::Error)
                            .message("Failed to load media file."),
                    ));
                }
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();
        {
            let margin = Margin::new_all_same(4.0);

            let mut bottom_bar = layout! {
                StackPanel::new(Orient::Horizontal),
                self.play_button   => { margin: margin },
                self.time_slider   => { margin: margin, grow: true },
                self.volume_slider => { margin: margin, width: 200.0 },
                self.volume_label  => { margin: margin, valign: VAlign::Center, halign: HAlign::Left, width: 20.0 },
                self.browse_button => { margin: margin }
            };
            let mut grid = layout! {
                Grid::from_str("1*", "1*,auto").unwrap(),
                self.media => { column: 0, row: 0 },
                bottom_bar => { column: 0, row: 1 },
            };
            grid.set_size(csize);
        }
    }
}

impl Deref for MediaPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
