pub mod error;
pub mod parser;
pub mod parser_new;

pub use error::{AppResult, KanskeError};

use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_registry};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

pub struct AppState {
    pub manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    pub heads: Vec<HeadInfo>,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct HeadInfo {
    pub head: zwlr_output_head_v1::ZwlrOutputHeadV1,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub current_mode: Option<ModeInfo>,
    pub modes: Vec<ModeInfo>,
}

#[derive(Debug, Clone)]
pub struct ModeInfo {
    pub mode: zwlr_output_mode_v1::ZwlrOutputModeV1,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "zwlr_output_manager_v1" {
                let manager = registry.bind::<zwlr_output_manager_v1::ZwlrOutputManagerV1, _, _>(
                    name,
                    version.min(4),
                    qh,
                    (),
                );
                state.manager = Some(manager);
            }
        }
    }
}

impl Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _: &zwlr_output_manager_v1::ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => {
                let head_info = HeadInfo {
                    head: head.clone(),
                    name: String::new(),
                    description: String::new(),
                    enabled: false,
                    current_mode: None,
                    modes: Vec::new(),
                };
                state.heads.push(head_info);
            }
            zwlr_output_manager_v1::Event::Done { serial } => {
                state.serial = Some(serial);
                println!("Manager done, serial: {}", serial);
            }
            zwlr_output_manager_v1::Event::Finished => {
                println!("Manager finished");
            }
            _ => {}
        }
    }

    wayland_client::event_created_child!(AppState, zwlr_output_manager_v1::ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (zwlr_output_head_v1::ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<zwlr_output_head_v1::ZwlrOutputHeadV1, ()> for AppState {
    fn event(
        state: &mut Self,
        head: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(head_info) = state.heads.iter_mut().find(|h| &h.head == head) {
            match event {
                zwlr_output_head_v1::Event::Name { name } => {
                    head_info.name = name;
                }
                zwlr_output_head_v1::Event::Description { description } => {
                    head_info.description = description;
                }
                zwlr_output_head_v1::Event::Enabled { enabled } => {
                    head_info.enabled = enabled != 0;
                }
                zwlr_output_head_v1::Event::CurrentMode { mode } => {
                    if let Some(mode_info) = head_info.modes.iter().find(|m| &m.mode == &mode) {
                        head_info.current_mode = Some(mode_info.clone());
                    }
                }
                zwlr_output_head_v1::Event::Mode { mode } => {
                    let mode_info = ModeInfo {
                        mode: mode.clone(),
                        width: 0,
                        height: 0,
                        refresh: 0,
                    };
                    head_info.modes.push(mode_info);
                }
                _ => {}
            }
        }
    }

    wayland_client::event_created_child!(AppState, zwlr_output_head_v1::ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (zwlr_output_mode_v1::ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<zwlr_output_mode_v1::ZwlrOutputModeV1, ()> for AppState {
    fn event(
        state: &mut Self,
        mode_obj: &zwlr_output_mode_v1::ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        for head in &mut state.heads {
            if let Some(mode_info) = head.modes.iter_mut().find(|m| &m.mode == mode_obj) {
                match event {
                    zwlr_output_mode_v1::Event::Size { width, height } => {
                        mode_info.width = width;
                        mode_info.height = height;
                    }
                    zwlr_output_mode_v1::Event::Refresh { refresh } => {
                        mode_info.refresh = refresh;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Dispatch<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _: &zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_configuration_v1::Event::Succeeded => {
                println!("Configuration succeeded!");
            }
            zwlr_output_configuration_v1::Event::Failed => {
                println!("Configuration failed!");
            }
            zwlr_output_configuration_v1::Event::Cancelled => {
                println!("Configuration cancelled!");
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
        _: zwlr_output_configuration_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
