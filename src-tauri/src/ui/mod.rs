pub mod tab_bar;
pub mod theme;
pub mod theming;

use iced::widget::{column, container};
use iced::{Element, Length, Application, Settings};
use crate::ui::theme::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tauri::Manager;
use iced::futures::SinkExt;

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
    ThemeChanged(String),
    TabTitleUpdated(String, String), // (tab_id, new_title)
    WindowMoved(i32, i32),
    WindowResized(u32, u32),
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
            Message::WindowMoved(x, y) => {
                if let Some(handle) = &self.app_handle {
                    if let Some(w) = handle.get_window("main") {
                        // Offset the engine window to account for the tab bar height (32px)
                        let _ = w.set_position(tauri::LogicalPosition::new(x as f64, (y as f64) + 32.0));
                    }
                }
            }
            Message::WindowResized(width, height) => {
                if let Some(handle) = &self.app_handle {
                    if let Some(w) = handle.get_window("main") {
                        // Resizing the engine window to match the shell minus tab bar
                        let _ = w.set_size(tauri::LogicalSize::new(width as f64, (height as f64) - 32.0));
                    }
                }
            }
            Message::TauriReady(handle) => {
                log::info!("Tauri is ready, handle received.");
                self.app_handle = Some(handle.clone());

                // Spawn all blocking initialization on a background thread
                // so the Iced UI thread stays responsive
                std::thread::spawn(move || {
                    // Wait for Tauri managed state to become available
                    let security = {
                        let mut attempts = 0;
                        loop {
                            let state = handle.try_state::<Arc<dyn crate::traits::SecuritySandbox>>();
                            if let Some(s) = state {
                                break s.inner().clone();
                            }
                            attempts += 1;
                            if attempts > 100 {
                                log::error!("SecuritySandbox state not available after 5s");
                                return;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    };

                    match crate::window_controller::WindowController::new(&handle, security) {
                        Ok(wc) => {
                            wc.setup_listeners(handle.clone());
                            if let Err(e) = wc.setup_tabs(&handle) {
                                log::error!("Failed to set up tabs: {}", e);
                            }
                            log::info!("WindowController initialized and set up.");
                            // Note: WindowController is not Send, so we can't send it back.
                            // It lives on this thread; the Tauri event loop keeps it alive.
                            std::mem::forget(wc);
                        }
                        Err(e) => {
                            log::error!("Failed to create WindowController: {}", e);
                        }
                    }
                });
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
            Message::ThemeChanged(theme_name) => {
                log::info!("UI: Theme changed to: {}", theme_name);
                if let Some(handle) = &self.app_handle {
                    let orchestrator = handle.state::<Arc<dyn crate::traits::TabOrchestrator>>().inner().clone();
                    let theming = handle.state::<Arc<dyn crate::traits::ThemingEngine>>().inner().clone();
                    
                    theming.set_active_theme(&theme_name);
                    
                    let tab_ids = orchestrator.get_tab_ids();
                    for tab_id in tab_ids {
                        let _ = orchestrator.inject_theme_into_tab(handle, &tab_id, &theme_name);
                    }
                }
            }
            Message::TabTitleUpdated(tab_id, new_title) => {
                for tab in &mut self.tabs {
                    if tab.id == tab_id {
                        tab.title = new_title.clone();
                        log::info!("UI: Tab {} title updated to: {}", tab_id, new_title);
                        break;
                    }
                }
            }
        }
        iced::Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let receiver = self.receiver.clone();
        
        iced::Subscription::batch(vec![
            iced::subscription::channel(
                std::any::TypeId::of::<()>(), 
                100, 
                move |mut _output| async move {
                    let rx = {
                        let mut rx_opt = receiver.lock().unwrap();
                        rx_opt.take()
                    };

                    if let Some(mut rx) = rx {
                        while let Some(message) = rx.recv().await {
                            let _ = _output.send(message).await;
                        }
                    }
                    std::future::pending().await
                }
            ),
            iced::event::listen().map(|event| match event {
                iced::Event::Window(_id, iced::window::Event::Moved { x, y }) => Message::WindowMoved(x, y),
                iced::Event::Window(_id, iced::window::Event::Resized { width, height }) => Message::WindowResized(width, height),
                _ => Message::Refresh, // Use Refresh as a no-op fallback
            }),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            tab_bar::view(&self.tabs),
            // Main content area placeholder
            container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(MainContentStyle)))
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct MainContentStyle;
impl container::StyleSheet for MainContentStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(LIGHT_BG_TAB_ACTIVE)),
            ..Default::default()
        }
    }
}

pub fn run(settings: Settings<Flags>) -> iced::Result {
    LotionApp::run(settings)
}
