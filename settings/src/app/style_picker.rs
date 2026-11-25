// SPDX-License-Identifier: GPL-3.0-only

//! Cards with a small wireframe to pick the menu style and the app layout,
//! after the layout picker of upstream's settings.

use cosmic::iced::core::text::EllipsizeHeightLimit;
use cosmic::iced::widget::text::{Ellipsize, Wrapping};
use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::widget::{Space, container, text, tooltip};
use cosmic::{Element, Theme};
use cosmic_ext_classic_menu_plus_applet::config::{MenuLayout, MenuStyle};

use super::Message;
use crate::fl;

pub(super) fn menu_style_picker(selected: MenuStyle) -> Element<'static, Message> {
    picker(None, MenuStyle::ALL.into_iter().map(|style| {
        let (name, description) = match style {
            MenuStyle::Classic => (fl!("style-classic"), fl!("style-classic-description")),
            MenuStyle::Compact => (fl!("style-compact"), fl!("style-compact-description")),
            MenuStyle::Sidebar => (fl!("style-sidebar"), fl!("style-sidebar-description")),
        };
        card(
            style_thumbnail(style),
            name,
            description,
            style == selected,
            Message::MenuStyleChanged(style),
        )
    }))
}

pub(super) fn layout_picker(selected: MenuLayout) -> Element<'static, Message> {
    picker(
        Some(fl!("layout")),
        [MenuLayout::Grid, MenuLayout::List, MenuLayout::Hybrid]
            .into_iter()
            .map(|layout| {
                let (name, description) = match layout {
                    MenuLayout::Grid => (fl!("layout-grid"), fl!("layout-grid-description")),
                    MenuLayout::List => (fl!("layout-list"), fl!("layout-list-description")),
                    MenuLayout::Hybrid => (fl!("layout-hybrid"), fl!("layout-hybrid-description")),
                };
                card(
                    layout_thumbnail(layout),
                    name,
                    description,
                    layout == selected,
                    Message::LayoutChanged(layout),
                )
            }),
    )
}

/// A row of cards, under `title` when given.
fn picker(
    title: Option<String>,
    cards: impl Iterator<Item = Element<'static, Message>>,
) -> Element<'static, Message> {
    let space = cosmic::theme::active().cosmic().spacing;
    let cards = cosmic::widget::row::with_children(cards.collect::<Vec<_>>())
        .spacing(space.space_xs)
        .width(Length::Fill);
    let mut content = cosmic::widget::column::with_capacity(2).spacing(space.space_xs);
    if let Some(title) = title {
        content = content.push(text::body(title));
    }
    container(content.push(cards))
        .padding([space.space_xs, space.space_m])
        .into()
}

fn card(
    thumbnail: Element<'static, Message>,
    name: String,
    description: String,
    selected: bool,
    on_press: Message,
) -> Element<'static, Message> {
    let space_xxs = cosmic::theme::active().cosmic().spacing.space_xxs;
    // Long translations get cut off on the card, so show them in full on hover.
    let full_text = format!("{name} · {description}");
    let card = cosmic::widget::button::custom(
        cosmic::iced::widget::column![
            thumbnail,
            text::body(name)
                .wrapping(Wrapping::None)
                .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1))),
            text::caption(description)
                .wrapping(Wrapping::None)
                .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1))),
        ]
        .spacing(space_xxs)
        .align_x(Alignment::Center)
        .width(Length::Fill),
    )
    .class(cosmic::theme::Button::Image)
    .selected(selected)
    .padding(space_xxs)
    .width(Length::FillPortion(1))
    .on_press(on_press);

    tooltip(card, text::caption(full_text), tooltip::Position::Bottom).into()
}

/// Wireframe of where a style puts the menu's parts.
fn style_thumbnail(style: MenuStyle) -> Element<'static, Message> {
    let bar = |height: f32| shape(Length::Fill, Length::Fixed(height), Kind::Line);
    let pane = |portion: u16| shape(Length::FillPortion(portion), Length::Fill, Kind::Pane);
    let accent_dot = || shape(Length::Fixed(5.0), Length::Fixed(5.0), Kind::Accent);

    let content: Element<'static, Message> = match style {
        MenuStyle::Classic => {
            // The power buttons sit under the categories.
            let categories = cosmic::iced::widget::column![
                pane(1),
                shape(Length::Fill, Length::Fixed(3.0), Kind::Accent),
            ]
            .spacing(2)
            .width(Length::FillPortion(3))
            .height(Length::Fill);
            cosmic::iced::widget::column![
                cosmic::iced::widget::row![accent_dot(), bar(5.0)].spacing(2),
                bar(5.0),
                cosmic::iced::widget::row![pane(5), categories]
                    .spacing(2)
                    .height(Length::Fill),
            ]
            .spacing(2)
            .into()
        }
        MenuStyle::Compact => cosmic::iced::widget::column![
            cosmic::iced::widget::row![accent_dot(), bar(5.0)].spacing(2),
            cosmic::iced::widget::row![pane(5), pane(3)]
                .spacing(2)
                .height(Length::Fill),
            cosmic::iced::widget::row![
                Space::new().width(Length::Fill),
                shape(Length::Fixed(20.0), Length::Fixed(3.0), Kind::Accent),
            ],
        ]
        .spacing(2)
        .into(),
        MenuStyle::Sidebar => {
            // Favorites at the top of the strip, power buttons at the bottom.
            let strip = cosmic::iced::widget::column![
                accent_dot(),
                shape(Length::Fixed(5.0), Length::Fixed(5.0), Kind::Line),
                shape(Length::Fixed(5.0), Length::Fixed(5.0), Kind::Line),
                Space::new().height(Length::Fill),
                shape(Length::Fixed(5.0), Length::Fixed(8.0), Kind::Accent),
            ]
            .spacing(2)
            .height(Length::Fill);
            cosmic::iced::widget::column![
                bar(5.0),
                cosmic::iced::widget::row![strip, pane(3), pane(5)]
                    .spacing(2)
                    .height(Length::Fill),
            ]
            .spacing(2)
            .into()
        }
    };

    frame(content)
}

/// Wireframe of how the app list shows apps.
fn layout_thumbnail(layout: MenuLayout) -> Element<'static, Message> {
    let tiles = || -> Element<'static, Message> {
        cosmic::widget::row::with_children(
            (0..4)
                .map(|_| shape(Length::Fixed(7.0), Length::Fixed(7.0), Kind::Accent))
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .into()
    };
    let item = || -> Element<'static, Message> {
        cosmic::iced::widget::row![
            shape(Length::Fixed(5.0), Length::Fixed(5.0), Kind::Accent),
            shape(Length::Fill, Length::Fixed(3.0), Kind::Line),
        ]
        .spacing(3)
        .align_y(Alignment::Center)
        .into()
    };

    let rows: Vec<Element<'static, Message>> = match layout {
        MenuLayout::Grid => (0..3).map(|_| tiles()).collect(),
        MenuLayout::List => (0..5).map(|_| item()).collect(),
        MenuLayout::Hybrid => vec![tiles(), tiles(), item(), item()],
    };

    frame(
        cosmic::widget::column::with_children(rows)
            .spacing(4)
            .align_x(Alignment::Center)
            .into(),
    )
}

fn frame(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content)
        .padding(4)
        .width(Length::Fixed(72.0))
        .height(Length::Fixed(56.0))
        .class(cosmic::theme::Container::custom(|theme: &Theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(Background::Color(cosmic.bg_color().into())),
                border: Border {
                    color: cosmic.bg_divider().into(),
                    width: 1.0,
                    radius: cosmic.radius_m().into(),
                },
                ..Default::default()
            }
        }))
        .into()
}

#[derive(Clone, Copy)]
enum Kind {
    Line,
    Pane,
    Accent,
}

fn shape(width: Length, height: Length, kind: Kind) -> Element<'static, Message> {
    container(Space::new())
        .width(width)
        .height(height)
        .class(cosmic::theme::Container::custom(move |theme: &Theme| {
            let cosmic = theme.cosmic();
            let color: Color = match kind {
                Kind::Line => {
                    let mut color: Color = cosmic.on_bg_color().into();
                    color.a *= 0.3;
                    color
                }
                Kind::Pane => cosmic.bg_component_color().into(),
                Kind::Accent => cosmic.accent_color().into(),
            };
            container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: 1.5.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}
