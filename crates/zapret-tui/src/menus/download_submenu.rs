use crate::menus::{Menu, Row};
use crate::state::screens::VersionTarget;
use crate::state::AppState;

/// The downloader's version picker.
///
/// The three versions are not a cycleable value on a row: the tag has to be
/// named, and a `Recommended → Latest → Recommended` loop cannot carry a tag. So
/// all three sit on the row at once and the one in use is the one marked, which
/// is why this menu has a value made of several parts rather than one string.
pub fn render(app: &AppState, is_zapret: bool) -> Menu {
    let menu_state = if is_zapret {
        app.download_zapret_menu
    } else {
        app.download_strategies_menu
    };
    let target = if is_zapret {
        &app.nfqws_target
    } else {
        &app.strat_target
    };

    let rec_ver = if is_zapret {
        zapret_fetch::ZAPRET_REC_VER.to_string()
    } else {
        zapret_fetch::STRAT_REC_VER[..7].to_string()
    };

    let tag_label = match target {
        VersionTarget::Tag(t) => rust_i18n::t!("val_tag_fmt").replace("{}", t),
        _ => rust_i18n::t!("val_tag").into_owned(),
    };

    let options = [
        (
            matches!(target, VersionTarget::Recommended),
            format!("{} ({})", rust_i18n::t!("val_rec"), rec_ver),
        ),
        (
            matches!(target, VersionTarget::Latest),
            rust_i18n::t!("val_latest").into_owned(),
        ),
        (matches!(target, VersionTarget::Tag(_)), tag_label),
    ];

    let version_value = options
        .iter()
        .map(|(current, label)| {
            if *current {
                format!("[● {label}]")
            } else {
                format!("[{label}]")
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    let rows = vec![
        Row::value(
            if is_zapret {
                rust_i18n::t!("menu_subdl_title_zapret").to_string()
            } else {
                rust_i18n::t!("menu_subdl_title_strat").to_string()
            },
            version_value,
        ),
        Row::new(rust_i18n::t!("menu_subdl_tag")),
        Row::new(rust_i18n::t!("menu_subdl_start")),
        Row::new(rust_i18n::t!("menu_subdl_back")),
    ];

    let title = if is_zapret {
        rust_i18n::t!("tui_title_download_zapret")
    } else {
        rust_i18n::t!("tui_title_download_strat")
    };

    Menu::new(title, rows).at(menu_state.index())
}
