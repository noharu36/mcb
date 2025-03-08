use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
    WebviewWindow
};
use tauri_nspanel::{
    cocoa::appkit::NSWindowCollectionBehavior, ManagerExt, WebviewWindowExt,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_clipboard_manager::ClipboardExt;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn show_panel(handle: &AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();

    panel.show();
}

fn hide_panel(handle: &AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();

    panel.order_out(None);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let ctrl_v_shortcut =
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
            let cmd_c_shortcut = Shortcut::new(Some(Modifiers::SUPER), Code::KeyC);
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&hide_i, &separator, &quit_i])?;

            let handle = app.handle();

            let window: WebviewWindow = handle.get_webview_window("main").unwrap();
            let panel = window.to_panel().unwrap();

            #[allow(non_upper_case_globals)]
            const NSFloatWindowLevel: i32 = 4;
            panel.set_level(NSFloatWindowLevel);

            #[allow(non_upper_case_globals)]
            const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;
            panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

            panel.set_collection_behaviour(
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces,
            );

            let clips = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

            // ショートカットキー
            handle.plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        println!("{:?}", shortcut);
                        if shortcut == &cmd_c_shortcut {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    println!("Cmd-C Pressed!");
                                    clips.lock().unwrap().push(app.clipboard().read_text().unwrap());
                                    println!("{:?}", clips);
                                    hide_panel(&app);
                                }
                                ShortcutState::Released => println!("Cmd-C Released!"),
                            }
                        } else if shortcut == &ctrl_v_shortcut {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    println!("Ctrl-V Pressed!");
                                    show_panel(&app);
                                }
                                ShortcutState::Released => println!("Ctrl-V Released!"),
                            }
                        }
                    })
                    .build(),
            )?;
            app.global_shortcut().register(ctrl_v_shortcut)?;
            app.global_shortcut().register(cmd_c_shortcut)?;

            // Macのメニューバー
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "hide" => {
                        println!("hide menu item was clicked");
                        app.hide().unwrap();
                    }
                    _ => println!("menu item {:?} not handled", event.id),
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
