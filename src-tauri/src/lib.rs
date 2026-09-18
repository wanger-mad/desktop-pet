use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, PhysicalPosition, RunEvent, State, WebviewWindow,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetSettings {
    size: u32,
    show_bubble: bool,
    click_through: bool,
    always_on_top: bool,
    auto_start: bool,
    focus_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
    long_break_every: u32,
    pomodoro_auto_start: bool,
    pomodoro_bubble: bool,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            size: 300,
            show_bubble: true,
            click_through: false,
            always_on_top: true,
            auto_start: false,
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_every: 4,
            pomodoro_auto_start: false,
            pomodoro_bubble: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetState {
    version: u32,
    affection: u32,
    energy: u32,
    hunger: u32,
    boredom: u32,
    interactions: u32,
    mood: String,
    settings: PetSettings,
    window_x: Option<i32>,
    window_y: Option<i32>,
    #[serde(default)]
    pomodoro: PomodoroState,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PomodoroState {
    phase: String,
    running: bool,
    remaining_seconds: u64,
    round: u32,
    total_completed: u32,
    end_epoch: Option<u64>,
}

impl Default for PomodoroState {
    fn default() -> Self { Self { phase: "idle".into(), running: false, remaining_seconds: 0, round: 1, total_completed: 0, end_epoch: None } }
}

impl Default for PetState {
    fn default() -> Self {
        Self {
            version: 1,
            affection: 52,
            energy: 78,
            hunger: 28,
            boredom: 20,
            interactions: 0,
            mood: "calm".into(),
            settings: PetSettings::default(),
            window_x: None,
            window_y: None,
            pomodoro: PomodoroState::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPatch {
    size: Option<u32>,
    show_bubble: Option<bool>,
    click_through: Option<bool>,
    always_on_top: Option<bool>,
    auto_start: Option<bool>,
    focus_minutes: Option<u32>,
    short_break_minutes: Option<u32>,
    long_break_minutes: Option<u32>,
    long_break_every: Option<u32>,
    pomodoro_auto_start: Option<bool>,
    pomodoro_bubble: Option<bool>,
}

struct Store {
    state: Mutex<PetState>,
    path: PathBuf,
}

fn load_state(path: &PathBuf) -> PetState {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .filter(|state: &PetState| state.version == 1)
        .unwrap_or_default()
}

fn save_state(store: &Store) {
    if let Ok(state) = store.state.lock() {
        if let Some(parent) = store.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*state) {
            let _ = fs::write(&store.path, json);
        }
    }
}

fn now_epoch() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

fn advance_pomodoro(state: &mut PetState) {
    if !state.pomodoro.running { return; }
    let now = now_epoch();
    let end = state.pomodoro.end_epoch.unwrap_or(now);
    if end > now { state.pomodoro.remaining_seconds = end - now; return; }
    let next = match state.pomodoro.phase.as_str() {
        "focus" => { state.pomodoro.total_completed += 1; if state.pomodoro.total_completed % state.settings.long_break_every == 0 { "longBreak" } else { "shortBreak" } },
        "shortBreak" | "longBreak" => "focus",
        _ => "idle",
    };
    if next == "idle" { state.pomodoro.running = false; state.pomodoro.end_epoch = None; state.pomodoro.remaining_seconds = 0; return; }
    if next == "focus" { state.pomodoro.round = if state.pomodoro.round >= state.settings.long_break_every { 1 } else { state.pomodoro.round + 1 }; }
    let mins = if next == "focus" { state.settings.focus_minutes } else if next == "shortBreak" { state.settings.short_break_minutes } else { state.settings.long_break_minutes };
    state.pomodoro.phase = next.into();
    state.pomodoro.remaining_seconds = (mins * 60) as u64;
    state.pomodoro.end_epoch = Some(now + state.pomodoro.remaining_seconds);
    if !state.settings.pomodoro_auto_start && next != "focus" { state.pomodoro.running = false; }
}

fn apply_window_settings(window: &WebviewWindow, state: &PetState) {
    let size = state.settings.size as f64;
    let _ = window.set_size(tauri::LogicalSize::new(size, size));
    let _ = window.set_always_on_top(state.settings.always_on_top);
    let _ = window.set_ignore_cursor_events(state.settings.click_through);
}

#[tauri::command]
fn get_state(store: State<Store>) -> PetState {
    let mut state = store.state.lock().expect("state lock poisoned");
    advance_pomodoro(&mut state);
    let result = state.clone(); drop(state); save_state(&store); result
}

#[tauri::command]
fn pomodoro_action(action: String, store: State<Store>) -> PetState {
    let mut state = store.state.lock().expect("state lock poisoned");
    advance_pomodoro(&mut state);
    match action.as_str() {
        "start" | "resume" => {
            if state.pomodoro.phase == "idle" || state.pomodoro.phase == "completed" { state.pomodoro.phase = "focus".into(); state.pomodoro.round = 1; state.pomodoro.remaining_seconds = state.settings.focus_minutes as u64 * 60; }
            state.pomodoro.running = true;
            state.pomodoro.end_epoch = Some(now_epoch() + state.pomodoro.remaining_seconds);
        },
        "pause" => { if state.pomodoro.running { state.pomodoro.running = false; state.pomodoro.end_epoch = None; } },
        "reset" => { state.pomodoro = PomodoroState::default(); },
        _ => {}
    }
    let result = state.clone(); drop(state); save_state(&store); result
}

#[tauri::command]
fn interact(kind: String, store: State<Store>) -> PetState {
    let mut state = store.state.lock().expect("state lock poisoned");
    state.interactions = state.interactions.saturating_add(1);
    state.affection = (state.affection + if kind == "head" { 2 } else { 1 }).min(100);
    state.boredom = state.boredom.saturating_sub(4);
    state.energy = state.energy.saturating_sub(1);
    state.mood = if state.energy < 20 {
        "sleepy"
    } else if state.interactions % 9 == 0 {
        "annoyed"
    } else {
        "happy"
    }
    .into();
    let result = state.clone();
    drop(state);
    save_state(&store);
    result
}

#[tauri::command]
fn update_settings(patch: SettingsPatch, app: AppHandle, store: State<Store>) -> PetState {
    let mut state = store.state.lock().expect("state lock poisoned");
    if let Some(size) = patch.size.filter(|v| [150, 300, 420, 500].contains(v)) {
        state.settings.size = size;
    }
    if let Some(value) = patch.show_bubble {
        state.settings.show_bubble = value;
    }
    if let Some(value) = patch.click_through {
        state.settings.click_through = value;
    }
    if let Some(value) = patch.always_on_top {
        state.settings.always_on_top = value;
    }
    if let Some(value) = patch.auto_start {
        state.settings.auto_start = value;
    }
    if let Some(value) = patch.focus_minutes { state.settings.focus_minutes = value.clamp(1, 120); }
    if let Some(value) = patch.short_break_minutes { state.settings.short_break_minutes = value.clamp(1, 60); }
    if let Some(value) = patch.long_break_minutes { state.settings.long_break_minutes = value.clamp(1, 60); }
    if let Some(value) = patch.long_break_every { state.settings.long_break_every = value.clamp(1, 10); }
    if let Some(value) = patch.pomodoro_auto_start { state.settings.pomodoro_auto_start = value; }
    if let Some(value) = patch.pomodoro_bubble { state.settings.pomodoro_bubble = value; }
    let result = state.clone();
    if let Some(window) = app.get_webview_window("pet") {
        apply_window_settings(&window, &result);
    }
    drop(state);
    save_state(&store);
    result
}

#[tauri::command]
fn reset_state(app: AppHandle, store: State<Store>) -> PetState {
    let current_settings = store
        .state
        .lock()
        .expect("state lock poisoned")
        .settings
        .clone();
    let mut state = PetState::default();
    state.settings = current_settings;
    *store.state.lock().expect("state lock poisoned") = state.clone();
    if let Some(window) = app.get_webview_window("pet") {
        apply_window_settings(&window, &state);
    }
    save_state(&store);
    state
}

#[tauri::command]
fn open_settings(app: AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_click_through(app: &AppHandle) {
    let store = app.state::<Store>();
    if let Ok(mut state) = store.state.lock() {
        state.settings.click_through = !state.settings.click_through;
        if let Some(window) = app.get_webview_window("pet") {
            let _ = window.set_ignore_cursor_events(state.settings.click_through);
        }
    }
    save_state(&store);
}

fn tray_pomodoro(action: &str, app: &AppHandle) {
    let store = app.state::<Store>();
    let mut state = store.state.lock().expect("state lock poisoned");
    advance_pomodoro(&mut state);
    match action {
        "start" => {
            if state.pomodoro.phase == "idle" || state.pomodoro.phase == "completed" { state.pomodoro.phase = "focus".into(); state.pomodoro.round = 1; state.pomodoro.remaining_seconds = state.settings.focus_minutes as u64 * 60; }
            state.pomodoro.running = true; state.pomodoro.end_epoch = Some(now_epoch() + state.pomodoro.remaining_seconds);
        },
        "pause" => { state.pomodoro.running = false; state.pomodoro.end_epoch = None; },
        "reset" => state.pomodoro = PomodoroState::default(),
        _ => {}
    }
    drop(state); save_state(&store);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("pet") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("state.json");
            let initial = load_state(&path);
            app.manage(Store {
                state: Mutex::new(initial.clone()),
                path,
            });

            if let Some(window) = app.get_webview_window("pet") {
                apply_window_settings(&window, &initial);
                if let (Some(x), Some(y)) = (initial.window_x, initial.window_y) {
                    let _ = window.set_position(PhysicalPosition::new(x, y));
                }
            }
            if let Some(settings_window) = app.get_webview_window("settings") {
                let to_hide = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = to_hide.hide();
                    }
                });
            }

            let settings = MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)?;
            let passthrough =
                MenuItem::with_id(app, "passthrough", "切换点击穿透", true, None::<&str>)?;
            let pomodoro_start = MenuItem::with_id(app, "pomodoro_start", "开始番茄钟", true, None::<&str>)?;
            let pomodoro_pause = MenuItem::with_id(app, "pomodoro_pause", "暂停番茄钟", true, None::<&str>)?;
            let pomodoro_reset = MenuItem::with_id(app, "pomodoro_reset", "重置番茄钟", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &pomodoro_start, &pomodoro_pause, &pomodoro_reset, &passthrough, &quit])?;
            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => open_settings(app.clone()),
                    "passthrough" => toggle_click_through(app),
                    "pomodoro_start" => tray_pomodoro("start", app),
                    "pomodoro_pause" => tray_pomodoro("pause", app),
                    "pomodoro_reset" => tray_pomodoro("reset", app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            pomodoro_action,
            interact,
            update_settings,
            reset_state,
            open_settings
        ]);

    builder
        .build(tauri::generate_context!())
        .expect("failed to build desktop pet")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                let store = app.state::<Store>();
                if let Some(window) = app.get_webview_window("pet") {
                    if let Ok(position) = window.outer_position() {
                        if let Ok(mut state) = store.state.lock() {
                            state.window_x = Some(position.x);
                            state.window_y = Some(position.y);
                        }
                    }
                }
                save_state(&store);
            }
        });
}
