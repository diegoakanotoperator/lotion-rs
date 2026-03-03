pub mod tab_bar;
pub mod theme;
pub mod theming;

use iced::widget::{column, container};
use iced::{Alignment, Element, Length, Sandbox, Settings};
use crate::ui::theme::*;

pub struct LotionApp {
    // Current state of the UI
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(String),
    NewTab,
    CloseTab(String),
    NavigateBack,
    NavigateForward,
    Refresh,
    Minimize,
    Maximize,
    CloseWindow,
}

impl Sandbox for LotionApp {
    type Message = Message;

    fn new() -> Self {
        Self {}
    }

    fn title(&self) -> String {
        String::from("Lotion")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TabSelected(tab_id) => {
                log::info!("UI: Tab selected: {}", tab_id);
            }
            Message::NewTab => {
                log::info!("UI: New tab requested");
            }
            Message::CloseTab(tab_id) => {
                log::info!("UI: Close tab: {}", tab_id);
            }
            Message::NavigateBack => {
                log::info!("UI: Navigate back");
            }
            Message::NavigateForward => {
                log::info!("UI: Navigate forward");
            }
            Message::Refresh => {
                log::info!("UI: Refresh");
            }
            Message::Minimize => {
                log::info!("UI: Minimize window");
            }
            Message::Maximize => {
                log::info!("UI: Maximize window");
            }
            Message::CloseWindow => {
                log::info!("UI: Close window");
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            tab_bar::view(),
            // Main content area (where webview will be positioned)
            container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Appearance {
                    background: Some(iced::Background::Color(LIGHT_BG_TAB_ACTIVE)),
                    ..Default::default()
                })
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub fn run() -> iced::Result {
    LotionApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            decorations: false, // Frameless window
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
