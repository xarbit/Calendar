use cosmic::iced::Length;
use cosmic::widget::{button, column, container, row, text, text_input};
use cosmic::{widget, Element};

use crate::app::{DeleteCalendarDialogState, NewCalendarDialogState};
use crate::components::color_picker::parse_hex_color;
use crate::fl;
use crate::message::Message;
use crate::styles::color_button_style;
use crate::ui_constants::{
    BORDER_WIDTH_HIGHLIGHT, BORDER_WIDTH_SELECTED, COLOR_BORDER_LIGHT, COLOR_BORDER_SELECTED,
    COLOR_BUTTON_SIZE_SMALL, COLOR_DEFAULT_GRAY, PADDING_STANDARD, SPACING_COLOR_GRID,
};

/// Render the new calendar dialog
pub fn render_new_calendar_dialog(state: &NewCalendarDialogState) -> Element<'_, Message> {
    let name_input = text_input(fl!("dialog-calendar-name-placeholder"), &state.name)
        .on_input(Message::NewCalendarNameChanged)
        .on_submit(|_| Message::ConfirmNewCalendar)
        .width(Length::Fill);

    // Color picker grid - same as quick_color_picker but for dialog
    let color_rows = [
        ["#3B82F6", "#0EA5E9", "#2563EB", "#1E40AF", "#06B6D4"],
        ["#8B5CF6", "#A78BFA", "#7C3AED", "#EC4899", "#DB2777"],
        ["#10B981", "#34D399", "#059669", "#14B8A6", "#0D9488"],
        ["#F59E0B", "#FBBF24", "#F97316", "#EF4444", "#DC2626"],
    ];

    let mut color_grid = column().spacing(SPACING_COLOR_GRID);

    for row_colors in color_rows {
        let mut color_row = row().spacing(SPACING_COLOR_GRID);

        for hex in row_colors {
            let color = parse_hex_color(hex).unwrap_or(COLOR_DEFAULT_GRAY);
            let hex_owned = hex.to_string();
            let is_selected = state.color == hex;

            let border_width = if is_selected {
                BORDER_WIDTH_SELECTED
            } else {
                BORDER_WIDTH_HIGHLIGHT
            };
            let border_color = if is_selected {
                COLOR_BORDER_SELECTED
            } else {
                COLOR_BORDER_LIGHT
            };

            let color_button = button::custom(
                container(widget::text(""))
                    .width(COLOR_BUTTON_SIZE_SMALL)
                    .height(COLOR_BUTTON_SIZE_SMALL)
                    .style(move |_theme: &cosmic::Theme| {
                        color_button_style(color, COLOR_BUTTON_SIZE_SMALL, border_width, border_color)
                    }),
            )
            .on_press(Message::NewCalendarColorChanged(hex_owned))
            .padding(0);

            color_row = color_row.push(color_button);
        }

        color_grid = color_grid.push(color_row);
    }

    // Dialog buttons
    let cancel_btn = button::text(fl!("button-cancel")).on_press(Message::CancelNewCalendar);

    let create_btn = button::suggested(fl!("button-create")).on_press(Message::ConfirmNewCalendar);

    let buttons = row()
        .spacing(8)
        .push(widget::horizontal_space())
        .push(cancel_btn)
        .push(create_btn);

    // Dialog content
    let content = column()
        .spacing(16)
        .push(text::title4(fl!("dialog-new-calendar-title")))
        .push(
            column()
                .spacing(8)
                .push(text(fl!("dialog-calendar-name")))
                .push(name_input),
        )
        .push(
            column()
                .spacing(8)
                .push(text(fl!("dialog-calendar-color")))
                .push(color_grid),
        )
        .push(buttons);

    // Dialog container with styling
    container(
        container(content)
            .padding(PADDING_STANDARD)
            .width(Length::Fixed(320.0))
            .style(|theme: &cosmic::Theme| {
                let cosmic = theme.cosmic();
                container::Style {
                    background: Some(cosmic::iced::Background::Color(cosmic.background.base.into())),
                    border: cosmic::iced::Border {
                        radius: cosmic.corner_radii.radius_m.into(),
                        width: 1.0,
                        color: cosmic.bg_divider().into(),
                    },
                    shadow: cosmic::iced::Shadow {
                        color: cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: cosmic::iced::Vector::new(0.0, 4.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                }
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme: &cosmic::Theme| container::Style {
        background: Some(cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
        ..Default::default()
    })
    .into()
}

/// Render the delete calendar confirmation dialog
pub fn render_delete_calendar_dialog(state: &DeleteCalendarDialogState) -> Element<'_, Message> {
    // Dialog buttons
    let cancel_btn = button::text(fl!("button-cancel")).on_press(Message::CancelDeleteCalendar);

    let delete_btn =
        button::destructive(fl!("button-delete")).on_press(Message::ConfirmDeleteCalendar);

    let buttons = row()
        .spacing(8)
        .push(widget::horizontal_space())
        .push(cancel_btn)
        .push(delete_btn);

    // Dialog content
    let content = column()
        .spacing(16)
        .push(text::title4(fl!("dialog-delete-calendar-title")))
        .push(text(fl!(
            "dialog-delete-calendar-message",
            name = state.calendar_name.clone()
        )))
        .push(buttons);

    // Dialog container with styling
    container(
        container(content)
            .padding(PADDING_STANDARD)
            .width(Length::Fixed(360.0))
            .style(|theme: &cosmic::Theme| {
                let cosmic = theme.cosmic();
                container::Style {
                    background: Some(cosmic::iced::Background::Color(cosmic.background.base.into())),
                    border: cosmic::iced::Border {
                        radius: cosmic.corner_radii.radius_m.into(),
                        width: 1.0,
                        color: cosmic.bg_divider().into(),
                    },
                    shadow: cosmic::iced::Shadow {
                        color: cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: cosmic::iced::Vector::new(0.0, 4.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                }
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme: &cosmic::Theme| container::Style {
        background: Some(cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
        ..Default::default()
    })
    .into()
}
