use kanske_lib::WaylandState;
use kanske_lib::parser::ast::Config;
use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::ZwlrOutputManagerV1;
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

pub struct KanskeState {
    pub wayland: WaylandState,
    pub config: Config,
    pub queue_handle: QueueHandle<KanskeState>,
    pub last_serial: Option<u32>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for KanskeState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qh: &QueueHandle<Self>,
    ) {
        state.wayland.handle_registry_event(registry, event, qh);
    }
}

impl Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for KanskeState {
    fn event(
        state: &mut Self,
        _: &ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _: &(),
        _: &wayland_client::Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.wayland.handle_manager_event(event);
    }

    wayland_client::event_created_child!(KanskeState, zwlr_output_manager_v1::ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (zwlr_output_head_v1::ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<zwlr_output_head_v1::ZwlrOutputHeadV1, ()> for KanskeState {
    fn event(
        state: &mut Self,
        head: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.wayland.handle_head_event(head, event);
    }

    wayland_client::event_created_child!(KanskeState, zwlr_output_head_v1::ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (zwlr_output_mode_v1::ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<zwlr_output_mode_v1::ZwlrOutputModeV1, ()> for KanskeState {
    fn event(
        state: &mut Self,
        mode_obj: &zwlr_output_mode_v1::ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.wayland.handle_mode_event(mode_obj, event);
    }
}

impl Dispatch<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1, ()> for KanskeState {
    fn event(
        state: &mut Self,
        _: &zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.wayland.handle_configuration_event(event);
    }
}

impl Dispatch<zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1, ()>
    for KanskeState
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
