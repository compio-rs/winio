use std::{path::PathBuf, time::Duration};

use compio::{runtime::spawn, time::interval};
use url::Url;
use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.media").run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    media: Child<Media>,
    playing: bool,
    play_button: Child<Button>,
    browse_button: Child<Button>,
    time_slider: Child<Slider>,
    volume_slider: Child<Slider>,
    volume_label: Child<Label>,
}

impl MainModel {
    fn set_playing(&mut self, v: bool) {
        self.playing = v;
        self.play_button
            .set_text(if self.playing { "⏸️" } else { "▶️" });
    }
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    Tick,
    Volume,
    Time,
    Play,
    ChooseFile,
    OpenFile(PathBuf),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Media example",
                size: Size::new(800.0, 600.0),
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
                minimum: 0,
            },
            volume_slider: Slider = (&window) => {
                enabled: false,
                minimum: 0,
                maximum: 100,
                pos: 100,
                freq: 20,
            },
            volume_label: Label = (&window),
        }
        sender.post(MainMessage::Volume);

        let sender = sender.clone();
        spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                sender.post(MainMessage::Tick);
            }
        })
        .detach();

        window.show();

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
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.volume_slider => {
                SliderEvent::Change => MainMessage::Volume,
            },
            self.time_slider => {
                SliderEvent::Change => MainMessage::Time,
            },
            self.play_button => {
                ButtonEvent::Click => MainMessage::Play,
            },
            self.browse_button => {
                ButtonEvent::Click => MainMessage::ChooseFile,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        self.window.update().await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Tick => {
                let ct = self.media.current_time();
                let ft = self.media.full_time();
                if let Some(ft) = ft {
                    let ft = ft.as_secs_f64();
                    self.time_slider.set_maximum((ft * 100.0) as _);
                    self.time_slider.set_pos((ct.as_secs_f64() * 100.0) as _);
                    self.time_slider.set_freq((ft * 100.0) as usize / 10);
                } else {
                    self.time_slider.set_maximum(1);
                    self.time_slider.set_pos(0);
                    self.time_slider.set_freq(1);
                }
                true
            }
            MainMessage::Volume => {
                let pos = self.volume_slider.pos();
                self.volume_label.set_text(pos.to_string());
                self.media.set_volume(pos as f64 / 100.0);
                true
            }
            MainMessage::Time => {
                let pos = self.time_slider.pos();
                let ft = self.media.full_time();
                if ft.is_some() {
                    self.media
                        .set_current_time(Duration::from_secs_f64(pos as f64 / 100.0));
                }
                true
            }
            MainMessage::Play => {
                if self.playing {
                    self.media.pause();
                    self.set_playing(false);
                } else {
                    self.media.play();
                    self.set_playing(true);
                }
                true
            }
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open media file")
                    .add_filter(("MP4 video", "*.mp4"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await
                {
                    sender.post(MainMessage::OpenFile(p));
                }
                false
            }
            MainMessage::OpenFile(p) => {
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
                    MessageBox::new()
                        .buttons(MessageBoxButton::Ok)
                        .style(MessageBoxStyle::Error)
                        .message("Failed to load media file.")
                        .show(&self.window)
                        .await;
                }
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let csize = self.window.client_size();
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
