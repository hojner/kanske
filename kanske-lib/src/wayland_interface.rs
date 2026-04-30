use tracing::{debug, info, trace, warn};

use crate::{AppResult, WaylandState};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, protocol::wl_registry};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

pub fn wayland_setup() -> AppResult<(
    WaylandState,
    Connection,
    EventQueue<WaylandState>,
    QueueHandle<WaylandState>,
)> {
    let wayland_connection = Connection::connect_to_env()?;
    debug!("Wayland connection established");
    let mut event_queue = wayland_connection.new_event_queue();
    let queue_handle: QueueHandle<WaylandState> = event_queue.handle();

    let mut state = WaylandState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };
    wayland_connection.display().get_registry(&queue_handle, ());

    // Decision: Making two roundtrips will give us the initial state of
    // the app. It's not pretty but gets the job done, and seems to be the
    // prefered option for many production apps. For now it's good
    // enough, might be handled in a while !ready loop later.
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;
    debug!(heads = state.heads.len(), serial = ?state.serial, "Initial roundtrip complete");

    Ok((state, wayland_connection, event_queue, queue_handle))
}

#[derive(Debug, Clone)]
pub struct HeadInfo {
    pub head: zwlr_output_head_v1::ZwlrOutputHeadV1,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub current_mode: Option<ModeInfo>,
    pub modes: Vec<ModeInfo>,
    pub position: Option<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct ModeInfo {
    pub mode: zwlr_output_mode_v1::ZwlrOutputModeV1,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

impl std::fmt::Display for HeadInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if self.enabled {
            if let Some(ref mode) = self.current_mode {
                write!(
                    f,
                    " using mode {}x{}@{:.1}Hz",
                    mode.width,
                    mode.height,
                    mode.refresh as f64 / 1000.0
                )?;
            }
            if let Some((x, y)) = self.position {
                write!(f, " in position {},{}", x, y)?;
            }
        } else {
            write!(f, " disabled")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
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
            && interface == "zwlr_output_manager_v1"
        {
            debug!(version, "Found output manager protocol");
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

impl Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for WaylandState {
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
                    head,
                    name: String::new(),
                    description: String::new(),
                    enabled: false,
                    current_mode: None,
                    modes: Vec::new(),
                    position: None,
                };
                state.heads.push(head_info);
            }
            zwlr_output_manager_v1::Event::Done { serial } => {
                debug!(serial, heads = state.heads.len(), "Manager done");
                state.serial = Some(serial);
            }
            zwlr_output_manager_v1::Event::Finished => {
                info!("Output manager finished");
            }
            _ => {}
        }
    }

    wayland_client::event_created_child!(WaylandState, zwlr_output_manager_v1::ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (zwlr_output_head_v1::ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<zwlr_output_head_v1::ZwlrOutputHeadV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        head: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let zwlr_output_head_v1::Event::Finished = event {
            let name = state
                .heads
                .iter()
                .find(|h| &h.head == head)
                .map(|h| h.name.clone());
            debug!(head = ?name, "Head removed");
            state.heads.retain(|h| &h.head != head);
            return;
        }

        if let Some(head_info) = state.heads.iter_mut().find(|h| &h.head == head) {
            match event {
                zwlr_output_head_v1::Event::Name { name } => {
                    trace!(name = %name, "Head name set");
                    head_info.name = name;
                }
                zwlr_output_head_v1::Event::Description { description } => {
                    trace!(description = %description, "Head description set");
                    head_info.description = description;
                }
                zwlr_output_head_v1::Event::Enabled { enabled } => {
                    let enabled = enabled != 0;
                    trace!(head = %head_info.name, enabled, "Head enabled changed");
                    head_info.enabled = enabled;
                }
                zwlr_output_head_v1::Event::CurrentMode { mode } => {
                    if let Some(mode_info) = head_info.modes.iter().find(|m| m.mode == mode) {
                        trace!(
                            head = %head_info.name,
                            width = mode_info.width,
                            height = mode_info.height,
                            "Head current mode set"
                        );
                        head_info.current_mode = Some(mode_info.clone());
                    }
                }
                zwlr_output_head_v1::Event::Mode { mode } => {
                    trace!(head = %head_info.name, "New mode added to head");
                    let mode_info = ModeInfo {
                        mode,
                        width: 0,
                        height: 0,
                        refresh: 0,
                    };
                    head_info.modes.push(mode_info);
                }
                zwlr_output_head_v1::Event::Position { x, y } => {
                    trace!(head = %head_info.name, x, y, "Head position set");
                    head_info.position = Some((x, y));
                }
                _ => {}
            }
        }
    }

    wayland_client::event_created_child!(WaylandState, zwlr_output_head_v1::ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (zwlr_output_mode_v1::ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<zwlr_output_mode_v1::ZwlrOutputModeV1, ()> for WaylandState {
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

impl Dispatch<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1, ()> for WaylandState {
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
                info!("Configuration applied successfully");
            }
            zwlr_output_configuration_v1::Event::Failed => {
                warn!("Configuration apply failed");
            }
            zwlr_output_configuration_v1::Event::Cancelled => {
                warn!("Configuration apply cancelled");
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1, ()>
    for WaylandState
{
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
