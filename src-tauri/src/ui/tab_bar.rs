use iced::widget::{button, row, text, container, image};
use iced::{Alignment, Element, Length, Color, Background, Border};
use crate::ui::Message;
use crate::ui::theme::*;

pub fn view() -> Element<'static, Message> {
    container(
        row![
            nav_controls(),
            tab_list(),
            window_controls(),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fixed(32.0))
    .style(container_style)
    .into()
}

fn container_style(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(LIGHT_BG_PRIMARY)),
        border: Border {
            color: LIGHT_BORDER_COLOR,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn nav_controls() -> Element<'static, Message> {
    row![
        app_logo(),
        nav_button("‹", Message::NavigateBack),
        nav_button("›", Message::NavigateForward),
        nav_button("↻", Message::Refresh),
    ]
    .spacing(4)
    .padding([0, 8])
    .align_items(Alignment::Center)
    .into()
}

fn app_logo() -> Element<'static, Message> {
    container(
        text("L")
            .size(14)
            .color(LIGHT_TEXT_PRIMARY)
    )
    .width(Length::Fixed(22.0))
    .height(Length::Fixed(22.0))
    .center_x()
    .center_y()
    .into()
}

fn nav_button(label: &str, msg: Message) -> Element<'static, Message> {
    button(
        container(text(label).size(16))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .center_x()
            .center_y()
    )
    .on_press(msg)
    .style(nav_button_style)
    .into()
}

fn nav_button_style(theme: &iced::Theme, status: button::Status) -> button::Appearance {
    match status {
        button::Status::Hovered => button::Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => button::Appearance {
            background: None,
            border: Border::default(),
            ..Default::default()
        },
    }
}

fn tab_list() -> Element<'static, Message> {
    row![
        tab("Notion", true),
        tab("Documentation", false),
        button(text("+").size(18))
            .on_press(Message::NewTab)
            .style(new_tab_button_style)
    ]
    .spacing(0)
    .width(Length::Fill)
    .align_items(Alignment::Center)
    .into()
}

fn tab(title: &str, active: bool) -> Element<'static, Message> {
    container(
        row![
            text("📄").size(14),
            text(title).size(13).color(LIGHT_TEXT_PRIMARY),
            if active { 
                Element::from(text("")) 
            } else {
                button(text("×").size(16))
                    .on_press(Message::CloseTab(title.to_string()))
                    .style(close_button_style)
                    .into()
            }
        ]
        .spacing(6)
        .padding([4, 10])
        .align_items(Alignment::Center)
    )
    .width(Length::Fixed(150.0))
    .height(Length::Fill)
    .style(move |_| tab_style(active))
    .into()
}

fn tab_style(active: bool) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(if active { LIGHT_BG_TAB_ACTIVE } else { LIGHT_BG_TAB })),
        border: Border {
            color: LIGHT_BORDER_COLOR,
            width: if active { 0.0 } else { 0.0 },
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn window_controls() -> Element<'static, Message> {
    row![
        window_control_button("−", Message::Minimize),
        window_control_button("□", Message::Maximize),
        window_control_button("×", Message::CloseWindow),
    ]
    .align_items(Alignment::Center)
    .into()
}

fn window_control_button(label: &str, msg: Message) -> Element<'static, Message> {
    button(
        container(text(label).size(12))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(32.0))
            .center_x()
            .center_y()
    )
    .on_press(msg)
    .style(window_control_style)
    .into()
}

fn window_control_style(theme: &iced::Theme, status: button::Status) -> button::Appearance {
    match status {
        button::Status::Hovered => button::Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            ..Default::default()
        },
        _ => button::Appearance {
            background: None,
            ..Default::default()
        },
    }
}

fn new_tab_button_style(theme: &iced::Theme, status: button::Status) -> button::Appearance {
    match status {
        button::Status::Hovered => button::Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            ..Default::default()
        },
        _ => button::Appearance {
            background: None,
            ..Default::default()
        },
    }
}

fn close_button_style(theme: &iced::Theme, status: button::Status) -> button::Appearance {
    match status {
        button::Status::Hovered => button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => button::Appearance {
            background: None,
            ..Default::default()
        },
    }
}
