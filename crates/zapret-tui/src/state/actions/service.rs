//! The service screen: install, start, stop, restart, uninstall.
//!
//! The rows on this screen depend on what was installed and whether it is
//! running, which is why the handler branches on both before it looks at the
//! cursor.

use zapret_wrapper::paths;
use zapret_wrapper::service;

use crate::state::screens::ActiveScreen;
use crate::state::AppState;

pub fn activate(app: &mut AppState) {
    let mgr_opt = service::get_detected_manager();

    if let Some(mgr) = mgr_opt {
        let mut action_taken = true;
        let res = if !app.service_installed {
            match app.service_menu_index {
                0 => {
                    if !app.check_dependencies() {
                        action_taken = false;
                        Ok(())
                    } else {
                        let exe_path = std::env::current_exe().map_err(|e| e.to_string());
                        match exe_path {
                            Ok(p) => {
                                let config_path = paths::config_path();
                                let cache_dir = paths::cache_dir();
                                mgr.install(&p, &config_path, &cache_dir).and_then(|_| mgr.start())
                            }
                            Err(e) => Err(e),
                        }
                    }
                }
                1 => {
                    app.active_screen = ActiveScreen::Main;
                    app.status_message = None;
                    action_taken = false;
                    Ok(())
                }
                _ => {
                    action_taken = false;
                    Ok(())
                }
            }
        } else if app.service_active {
            match app.service_menu_index {
                0 => mgr.stop(),
                1 => {
                    if !app.check_dependencies() {
                        action_taken = false;
                        Ok(())
                    } else {
                        mgr.restart()
                    }
                }
                2 => mgr.uninstall(),
                3 => {
                    app.active_screen = ActiveScreen::Main;
                    app.status_message = None;
                    action_taken = false;
                    Ok(())
                }
                _ => {
                    action_taken = false;
                    Ok(())
                }
            }
        } else {
            match app.service_menu_index {
                0 => mgr.start(),
                1 => mgr.uninstall(),
                2 => {
                    app.active_screen = ActiveScreen::Main;
                    app.status_message = None;
                    action_taken = false;
                    Ok(())
                }
                _ => {
                    action_taken = false;
                    Ok(())
                }
            }
        };

        if action_taken {
            match res {
                Ok(_) => {
                    app.refresh_service_status();
                    app.service_menu_index = 0;
                    app.status_message = Some(rust_i18n::t!("msg_op_ok").into_owned());
                }
                Err(e) => {
                    app.refresh_service_status();
                    app.show_error(e);
                }
            }
        }
    } else {
        app.status_message = Some(rust_i18n::t!("msg_err_init").into_owned());
    }
}
