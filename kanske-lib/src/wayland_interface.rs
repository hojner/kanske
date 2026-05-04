use tracing::{debug, info, trace, warn};

use crate::AppResult;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, protocol::wl_registry};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

#[derive(Debug)]
pub struct WaylandState {
    pub manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    pub heads: Vec<HeadInfo>,
    pub serial: Option<u32>,
}

impl WaylandState {
    pub fn handle_registry_event<D>(
        &mut self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        qh: &QueueHandle<D>,
    ) where
        D: Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> + 'static,
    {
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
            self.manager = Some(manager);
        }
    }

    pub fn handle_manager_event(&mut self, event: zwlr_output_manager_v1::Event) {
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
                self.heads.push(head_info);
            }
            zwlr_output_manager_v1::Event::Done { serial } => {
                debug!(serial, heads = self.heads.len(), "Manager done");
                self.serial = Some(serial);
            }
            zwlr_output_manager_v1::Event::Finished => {
                info!("Output manager finished");
                self.manager = None;
            }
            _ => {}
        }
    }

    pub fn handle_head_event(
        &mut self,
        head: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
    ) {
        if let zwlr_output_head_v1::Event::Finished = event {
            let name = self
                .heads
                .iter()
                .find(|h| &h.head == head)
                .map(|h| h.name.clone());
            debug!(head = ?name, "Head removed");
            self.heads.retain(|h| &h.head != head);
            return;
        }

        if let Some(head_info) = self.heads.iter_mut().find(|h| &h.head == head) {
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

    pub fn handle_mode_event(
        &mut self,
        mode_obj: &zwlr_output_mode_v1::ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
    ) {
        for head in &mut self.heads {
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

    pub fn handle_configuration_event(&mut self, event: zwlr_output_configuration_v1::Event) {
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
        writeln!(f)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModeInfo {
    pub mode: zwlr_output_mode_v1::ZwlrOutputModeV1,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
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
        state.handle_registry_event(registry, event, qh);
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
        state.handle_manager_event(event);
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
        state.handle_head_event(head, event);
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
        state.handle_mode_event(mode_obj, event);
    }
}

impl Dispatch<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.handle_configuration_event(event);
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

pub fn connect<D>() -> AppResult<(Connection, EventQueue<D>, QueueHandle<D>)>
where
    D: Dispatch<wl_registry::WlRegistry, ()> + 'static,
{
    let connection = Connection::connect_to_env()?;
    debug!("Wayland connection established");
    let event_queue = connection.new_event_queue::<D>();
    let qh = event_queue.handle();
    connection.display().get_registry(&qh, ());
    Ok((connection, event_queue, qh))
}
