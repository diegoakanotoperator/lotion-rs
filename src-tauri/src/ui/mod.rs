pub mod tab_bar;
pub mod theme;
pub mod theming;

use iced::widget::{column, container};
use iced::{Alignment, Element, Length, Application, Settings};
use crate::ui::theme::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub active: bool,
}

pub struct LotionApp {
    pub tabs: Vec<TabInfo>,
    pub app_handle: Option<tauri::AppHandle>,
    pub receiver: Arc<Mutex<Option<mpsc::Receiver<Message>>>>,
    pub window_controller: Option<crate::window_controller::WindowController>,
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
    TauriReady(tauri::AppHandle),
}

pub struct Flags {
    pub rx: mpsc::Receiver<Message>,
}

impl iced::Application for LotionApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = Flags;

    fn new(flags: Flags) -> (Self, iced::Command<Message>) {
        let receiver = Arc::new(Mutex::new(Some(flags.rx)));
        (
            Self {
                tabs: vec![TabInfo {
                    id: "initial".to_string(),
                    title: "Notion".to_string(),
                    active: true,
                }],
                app_handle: None,
                receiver,
                window_controller: None,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Lotion")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::TauriReady(handle) => {
                log::info!("Tauri is ready, handle received.");
                self.app_handle = Some(handle.clone());

                // Retrieve the security module from Tauri's managed state
                let security = handle.state::<Arc<dyn crate::traits::SecuritySandbox>>().clone();

                match crate::window_controller::WindowController::new(&handle, security) {
                    Ok(wc) => {
                        wc.setup_listeners(handle.clone());
                        if let Err(e) = wc.setup_tabs(&handle) {
                            log::error!("Failed to set up tabs: {}", e);
                        }
                        self.window_controller = Some(wc);
                        log::info!("WindowController initialized and set up.");
                    }
                    Err(e) => {
                        log::error!("Failed to create WindowController: {}", e);
                    }
                }
            }
            Message::TabSelected(tab_id) => {
                for tab in &mut self.tabs {
                    tab.active = tab.id == tab_id;
                }
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
        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let receiver = self.receiver.clone();
        
        iced::subscription::channel(
            std::any::TypeId::of::<()>(), 
            100, 
            move |mut _output| async move {
                let mut rx_opt = receiver.lock().unwrap();
                if let Some(mut rx) = rx_opt.take() {
                    while let Some(message) = rx.recv().await {
                        let _ = _output.send(message).await;
                    }
                }
                std::future::pending().await
            }
        )
    }

    fn view(&self) -> Element<Message> {
// ... existing view ...
        let content = column![
            tab_bar::view(&self.tabs),
            // Main content area placeholder
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

pub fn run(settings: Settings<()>) -> iced::Result {
    LotionApp::run(settings)
}
