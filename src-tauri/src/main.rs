// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bluer::Adapter;
use bluer::AdapterEvent;
use bluer::Address;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use std::collections::HashMap;
use std::process::exit;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use std::vec;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use bluer::DiscoveryFilter;
use bluer::DiscoveryTransport;
use bluer::Session;
use serde::Serialize;
use tauri::AppHandle;
use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTray;
use tauri::SystemTrayEvent;
use tauri::SystemTrayMenu;
use tokio::runtime::Runtime;

type Devices = HashMap<Address, Device>;

#[derive(Debug, Serialize, Clone)]
struct Device {
    mac_addr: Address,
    name: String,
}

struct BluerClient {
    session: Session,
    rt: Runtime,
}
#[derive(Debug, Clone, Copy)]
enum Command {
    RefreshDevices,
    GetCurrentDevices,
    StopRefreshDevices,
}

struct AppState {
    devices: Arc<Mutex<HashMap<Address, Device>>>,
    background_controller_tx: Sender<Command>,
}
#[tauri::command]
fn get_devices(state: tauri::State<AppState>) -> Vec<Device> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            state
                .devices
                .lock()
                .await
                .clone()
                .values()
                .map(|x| x.clone())
                .collect()
        })
}

#[tauri::command]
fn get_device_state(state: tauri::State<AppState>) {}

#[tauri::command]
fn refresh_devices(state: tauri::State<AppState>) -> bool {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            state
                .background_controller_tx
                .send(Command::RefreshDevices)
                .await
                .is_ok()
        })
}

fn systme_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            system_tray_menu_item::id::QUIT => exit(0),
            system_tray_menu_item::id::DASHBOARD => {
                let window = app.get_window("main").expect("failed to get window");
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            system_tray_menu_item::id::QUICK_CONNECT => {
                use std::io::Write;
                let mut child = std::process::Command::new("bluetoothctl")
                    .stdin(Stdio::piped())
                    .spawn()
                    .unwrap();
                let mut stdin = child.stdin.take().unwrap();
                stdin.write("connect 48:73:CB:41:50:F5".as_bytes()).unwrap();
                stdin.flush().unwrap();
            }
            system_tray_menu_item::id::QUICK_DISCONNECT => {
                use std::io::Write;
                let mut child = std::process::Command::new("bluetoothctl")
                    .stdin(Stdio::piped())
                    .spawn()
                    .unwrap();
                let mut stdin = child.stdin.take().unwrap();
                stdin
                    .write("disconnect 48:73:CB:41:50:F5".as_bytes())
                    .unwrap();
                stdin.flush().unwrap();
            }
            _ => {}
        },
        _ => {}
    }
}

mod system_tray_menu_item {
    pub mod id {
        pub const QUIT: &str = "quit";
        pub const QUICK_CONNECT: &str = "quick connect";
        pub const QUICK_DISCONNECT: &str = "quick disconnect";
        pub const DASHBOARD: &str = "dashboard";
    }
    pub mod title {
        pub const QUIT: &str = "Quit";
        pub const QUICK_CONNECT: &str = "Quick connect";
        pub const QUICK_DISCONNECT: &str = "Quick disconnect";
        pub const DASHBOARD: &str = "Dashboard";
    }
}

fn background_thread(devices_ref: Arc<Mutex<Devices>>, rx_command: Receiver<Command>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to make a tokio runtime");

    rt.block_on(blockground_procedure(devices_ref, rx_command));
}

async fn handle_adapter_event(
    devices_ref: Arc<Mutex<Devices>>,
    adapter: Arc<Mutex<Adapter>>,
    e: AdapterEvent,
) {
    match e {
        AdapterEvent::DeviceAdded(addr) => {
            println!("device added: {}", addr);
            let device = match adapter.lock().await.device(addr) {
                Err(e) => return,
                Ok(d) => d,
            };
            devices_ref.lock().await.insert(
                addr,
                Device {
                    mac_addr: addr,
                    name: device
                        .name()
                        .await
                        .unwrap_or_default()
                        .unwrap_or("unknown".to_string()),
                },
            );
        }
        AdapterEvent::DeviceRemoved(addr) => {
            println!("device removed: {}", addr);
            devices_ref.lock().await.remove(&addr);
        }
        AdapterEvent::PropertyChanged(p) => {}
    }
}

async fn blockground_procedure(
    devices_ref: Arc<Mutex<Devices>>,
    mut rx_command: Receiver<Command>,
) {
    let session = Session::new().await.unwrap();
    let adapter = Arc::new(Mutex::new(session.default_adapter().await.unwrap()));

    let is_refresh_devices = Arc::new(Mutex::new(false));
    loop {
        println!("in loop");
        let c = match rx_command.recv().await {
            Some(c) => c,
            None => return,
        };
        println!("command: {:?}", c);
        let mut backgound_handlers = vec![];
        match c {
            Command::RefreshDevices => {
                {
                    //drop guard variable
                    let mut is_refresh_devices_guard = is_refresh_devices.lock().await;
                    if *is_refresh_devices_guard {
                        continue;
                    } else {
                        *is_refresh_devices_guard = true;
                    }
                }
                println!("before: thread spawn");
                let devices_ref_tmp = devices_ref.clone();
                let is_refresh_devices_tmp = is_refresh_devices.clone();
                let adapter_tmp = adapter.clone();
                let thread_handler = tokio::spawn(refresh_devices_background(
                    devices_ref_tmp,
                    is_refresh_devices_tmp,
                    adapter_tmp,
                ));
                backgound_handlers.push(thread_handler);
            }
            Command::GetCurrentDevices => {}
            Command::StopRefreshDevices => {}
        }
    }
}

async fn refresh_devices_background(
    devices_ref: Arc<Mutex<Devices>>,
    is_refresh_devices: Arc<Mutex<bool>>,
    adapter: Arc<Mutex<Adapter>>,
) {
    println!("before is_discovering");
    let is_discovering = match adapter.lock().await.is_discovering().await {
        Ok(b) => b,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    println!("after is_discovering");
    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Auto,
        ..Default::default()
    };
    adapter
        .lock()
        .await
        .set_discovery_filter(filter)
        .await
        .unwrap();
    if !is_discovering {
        let _ = match adapter.lock().await.discover_devices().await {
            Ok(e) => {}
            Err(e) => {}
        };
    }
    println!("before appending exists devices");
    append_exists_devices(devices_ref.clone(), adapter.clone()).await;
    println!("after appending exists devices");
    println!("length of devices: {}", devices_ref.lock().await.len());
    let mut events = adapter.lock().await.events().await.unwrap();
    loop {
        if !*is_refresh_devices.lock().await {
            return;
        }
        if let Some(event) = events.next().await {
            handle_adapter_event(devices_ref.clone(), adapter.clone(), event).await;
        } else {
            //no more events
            return;
        }
    }
}

async fn append_exists_devices(devices_ref: Arc<Mutex<Devices>>, adapter: Arc<Mutex<Adapter>>) {
    let addresses = adapter.lock().await.device_addresses().await.unwrap();
    for addr in addresses {
        let device = match adapter.lock().await.device(addr) {
            Ok(d) => d,
            Err(_) => {
                continue;
            }
        };
        devices_ref.lock().await.insert(
            addr,
            Device {
                mac_addr: addr,
                name: device
                    .name()
                    .await
                    .unwrap_or_default()
                    .unwrap_or("unknown".to_string()),
            },
        );
    }
}
fn main() {
    let (tx, rx) = channel::<Command>(1);
    let state = AppState {
        devices: Arc::new(Mutex::new(HashMap::new())),
        background_controller_tx: tx,
    };
    let devices_ref = state.devices.clone();
    thread::spawn(move || {
        background_thread(devices_ref, rx);
    });
    let quit = CustomMenuItem::new(
        system_tray_menu_item::id::QUIT,
        system_tray_menu_item::title::QUIT,
    );
    let dashboard = CustomMenuItem::new(
        system_tray_menu_item::id::DASHBOARD,
        system_tray_menu_item::title::DASHBOARD,
    );
    let quick_connect = CustomMenuItem::new(
        system_tray_menu_item::id::QUICK_CONNECT,
        system_tray_menu_item::title::QUICK_CONNECT,
    );
    let quick_disconnect = CustomMenuItem::new(
        system_tray_menu_item::id::QUICK_DISCONNECT,
        system_tray_menu_item::title::QUICK_DISCONNECT,
    );
    let tray_menu = SystemTrayMenu::new()
        .add_item(dashboard)
        .add_item(quick_connect)
        .add_item(quick_disconnect)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .setup(|app| {
            app.emit_all("", ()).unwrap();
            Ok(())
        })
        .system_tray(system_tray)
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_devices,
            refresh_devices,
            get_device_state
        ])
        .on_system_tray_event(systme_tray_event)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to start prorgam");
}
