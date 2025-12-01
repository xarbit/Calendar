//! Month view header rendering
//!
//! Contains the weekday header row rendering logic.

use cosmic::iced::{alignment, Length};
use cosmic::widget::{container, row};
use cosmic::{widget, Element};

use crate::fl;
use crate::localized_names;
use crate::message::Message;
use crate::ui_constants::{FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PADDING_SMALL, SPACING_TINY, WEEK_NUMBER_WIDTH};

/// Render the weekday header row with responsive names
pub fn render_weekday_header(show_week_numbers: bool, use_short_names: bool) -> Element<'static, Message> {
    let mut header_row = row().spacing(SPACING_TINY);

    // Week number header (only if enabled)
    if show_week_numbers {
        header_row = header_row.push(
            container(widget::text(fl!("week-abbr")).size(FONT_SIZE_SMALL))
                .width(Length::Fixed(WEEK_NUMBER_WIDTH))
                .padding(PADDING_SMALL)
                .align_y(alignment::Vertical::Center)
        );
    }

    // Weekday headers - use short or full names based on available width
    let weekday_names = if use_short_names {
        localized_names::get_weekday_names_short()
    } else {
        localized_names::get_weekday_names_full()
    };

    for weekday in weekday_names {
        header_row = header_row.push(
            container(widget::text(weekday).size(FONT_SIZE_MEDIUM))
                .width(Length::Fill)
                .padding(PADDING_SMALL)
                .center_x(Length::Fill),
        );
    }

    header_row.into()
}
