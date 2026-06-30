use iced::{Background, color, widget};

const BACKGROUND_COLOR: iced::Color = color!(0x0f172a);

const PRIMARY_COLOR: iced::Color = color!(0x1e293b);

const INPUT_BACKGROUND_COLOR: iced::Color = color!(0x334155);

const ACCENT_COLOR: iced::Color = color!(0x84cc16);
const ACCENT_HOVER_COLOR: iced::Color = color!(0x65a30d);
const ACCENT_TEXT_COLOR: iced::Color = color!(0x0f172a);

const TEXT_COLOR: iced::Color = color!(0xf8fafc);
const BORDER_COLOR: iced::Color = color!(0x334155);

const ROUNDNESS: f32 = 4.0;
pub const MIN_SECTION_SIZE: f32 = 30.0;
pub const LINE_WIDTH: f32 = 1.0;

pub fn container_whole_style() -> widget::container::Style {
    widget::container::Style::default()
        .color(TEXT_COLOR)
        .background(BORDER_COLOR)
        .border(
            iced::Border::default()
                .rounded(ROUNDNESS)
                .width(LINE_WIDTH)
                .color(BORDER_COLOR),
        )
}
/// Style for non interactable containers such as header, sidebar, primary
pub fn container_style() -> widget::container::Style {
    widget::container::Style::default()
        .color(TEXT_COLOR)
        .background(PRIMARY_COLOR)
        .border(
            iced::Border::default()
                .rounded(ROUNDNESS)
                .width(LINE_WIDTH)
                .color(BORDER_COLOR),
        )
}

/// Container style for interactable widgets: buttons, text inputs, etc
pub fn container_interactive_style() -> widget::container::Style {
    widget::container::Style::default()
        .color(TEXT_COLOR)
        .background(INPUT_BACKGROUND_COLOR)
        .border(
            iced::Border::default()
                .rounded(ROUNDNESS)
                .width(LINE_WIDTH)
                .color(BORDER_COLOR),
        )
}

pub fn button_style(status: widget::button::Status) -> widget::button::Style {
    let (bg, text) = match status {
        widget::button::Status::Active => (ACCENT_COLOR, ACCENT_TEXT_COLOR),
        widget::button::Status::Hovered => (ACCENT_HOVER_COLOR, ACCENT_TEXT_COLOR),
        widget::button::Status::Pressed => (ACCENT_HOVER_COLOR, ACCENT_TEXT_COLOR),
        widget::button::Status::Disabled => (color!(0xd1d5db), color!(0x9ca3af)),
    };

    widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: text,
        border: iced::Border::default().rounded(ROUNDNESS).width(0.0),
        ..Default::default()
    }
}
pub fn text_input_style(status: widget::text_input::Status) -> widget::text_input::Style {
    let border_color = match status {
        widget::text_input::Status::Focused { is_hovered: _ } => ACCENT_COLOR,
        _ => BORDER_COLOR,
    };

    widget::text_input::Style {
        background: iced::Background::Color(INPUT_BACKGROUND_COLOR),
        border: iced::Border::default()
            .rounded(ROUNDNESS)
            .width(LINE_WIDTH)
            .color(border_color),
        placeholder: color!(0x9ca3af),
        value: TEXT_COLOR,
        selection: color!(0xbfdbfe),
        icon: color!(0x9ca3af),
    }
}
pub fn text_style() -> widget::text::Style {
    widget::text::Style {
        color: Some(TEXT_COLOR),
    }
}
