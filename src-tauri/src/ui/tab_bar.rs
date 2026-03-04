use iced::widget::{button, row, text, container};
use iced::widget::button::Appearance;
use iced::{Alignment, Element, Length, Color, Background, Border, Theme};
use crate::ui::Message;
use crate::ui::theme::*;

pub fn view(tabs: &[crate::ui::TabInfo]) -> Element<'static, Message> {
    container(
        row![
            nav_controls(),
            tab_list(tabs),
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

fn container_style(_theme: &Theme) -> container::Appearance {
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
            .style(iced::theme::Text::Color(LIGHT_TEXT_PRIMARY))
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
    .style(iced::theme::Button::custom(NavButtonStyle))
    .into()
}

struct NavButtonStyle;
impl button::StyleSheet for NavButtonStyle {
    type Style = Theme;
    fn active(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: None,
            border: Border::default(),
            ..Default::default()
        }
    }
    fn hovered(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn tab_list(tabs: &[crate::ui::TabInfo]) -> Element<'static, Message> {
    let mut tab_row = row![].spacing(0).width(Length::Fill).align_items(Alignment::Center);
    
    for tab_info in tabs {
        tab_row = tab_row.push(tab(&tab_info.title, tab_info.active, &tab_info.id));
    }

    tab_row.push(
        button(text("+").size(18))
            .on_press(Message::NewTab)
            .style(iced::theme::Button::custom(NewTabButtonStyle))
    ).into()
}

struct NewTabButtonStyle;
impl button::StyleSheet for NewTabButtonStyle {
    type Style = Theme;
    fn active(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: None,
            ..Default::default()
        }
    }
    fn hovered(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            ..Default::default()
        }
    }
}

fn tab(title: &str, active: bool, id: &str) -> Element<'static, Message> {
    container(
        row![
            text("📄").size(14),
            text(title).size(13).style(iced::theme::Text::Color(LIGHT_TEXT_PRIMARY)),
            if active { 
                Element::from(text("")) 
            } else {
                button(text("×").size(16))
                    .on_press(Message::CloseTab(id.to_string()))
                    .style(iced::theme::Button::custom(CloseButtonStyle))
                    .into()
            }
        ]
        .spacing(6)
        .padding([4, 10])
        .align_items(Alignment::Center)
    )
    .width(Length::Fixed(150.0))
    .height(Length::Fill)
    .style(iced::theme::Container::Custom(Box::new(TabContainerStyle { active })))
    .into()
}

struct TabContainerStyle {
    active: bool,
}

impl container::StyleSheet for TabContainerStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(if self.active { LIGHT_BG_TAB_ACTIVE } else { LIGHT_BG_TAB })),
            border: Border {
                color: LIGHT_BORDER_COLOR,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
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
    .style(iced::theme::Button::custom(WindowControlStyle))
    .into()
}

struct WindowControlStyle;
impl button::StyleSheet for WindowControlStyle {
    type Style = Theme;
    fn active(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: None,
            ..Default::default()
        }
    }
    fn hovered(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: Some(Background::Color(LIGHT_BG_TAB)),
            ..Default::default()
        }
    }
}

struct CloseButtonStyle;
impl button::StyleSheet for CloseButtonStyle {
    type Style = Theme;
    fn active(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: None,
            ..Default::default()
        }
    }
    fn hovered(&self, _theme: &Theme) -> Appearance {
        Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.1))),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
