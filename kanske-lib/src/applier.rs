use std::collections::HashSet;

use wayland_client::{QueueHandle, protocol::wl_output};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
    zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
    zwlr_output_head_v1::AdaptiveSyncState,
};

use crate::{
    AppResult, KanskeState,
    error::KanskeError,
    matcher::find_matching_profile,
    parser::ast::{Config, OutputCommand, OutputConfig, OutputDesc},
    wayland_interface::HeadInfo,
};

pub fn find_and_apply_profile(
    state: &mut KanskeState,
    qh: &QueueHandle<KanskeState>,
    config: &Config,
) -> AppResult<()> {
    if let Some(profile) = find_matching_profile(&state.heads, config) {
        let mut used_indicies: HashSet<usize> = HashSet::new();
        let manager = state
            .manager
            .as_ref()
            .ok_or(KanskeError::ManagerNotAvailable)?;
        let serial = state.serial.ok_or(KanskeError::NoConfiguration)?;
        let output_configuration = manager.create_configuration(serial, qh, ());

        for output in profile
            .outputs
            .iter()
            .filter(|f| matches!(f.desc, OutputDesc::Name(_)))
        {
            let position = state
                .heads
                .iter()
                .enumerate()
                .find(|(i, h)| !used_indicies.contains(i) && output.desc.matches(&h.name))
                .map(|(i, _)| i)
                .ok_or(KanskeError::NoConfiguration)?;
            used_indicies.insert(position);
            let current_head = &state.heads[position];
            configure_head(output, &output_configuration, current_head, qh)?;
        }
        for output in profile
            .outputs
            .iter()
            .filter(|f| matches!(f.desc, OutputDesc::Any))
        {
            let position = (0..state.heads.len())
                .find(|i| !used_indicies.contains(i))
                .ok_or(KanskeError::NoConfiguration)?;
            used_indicies.insert(position);
            let current_head = &state.heads[position];
            configure_head(output, &output_configuration, current_head, qh)?;
        }
        output_configuration.apply();
    } else {
        return Ok(());
    }

    Ok(())
}

fn configure_head(
    output: &OutputConfig,
    output_configuration: &ZwlrOutputConfigurationV1,
    current_head: &HeadInfo,
    qh: &QueueHandle<KanskeState>,
) -> AppResult<()> {
    let is_enabled = output
        .commands
        .iter()
        .find_map(|c| {
            if let OutputCommand::Enabled(b) = c {
                Some(*b)
            } else {
                None
            }
        })
        .unwrap_or(true);
    if is_enabled {
        let head_config = output_configuration.enable_head(&current_head.head, qh, ());
        for command in output.commands.iter() {
            apply_command(&head_config, command, current_head)?;
        }
    } else {
        output_configuration.disable_head(&current_head.head);
    }
    Ok(())
}

fn apply_command(
    head_config: &ZwlrOutputConfigurationHeadV1,
    command: &OutputCommand,
    current_head: &HeadInfo,
) -> AppResult<()> {
    match command {
        OutputCommand::Mode {
            width,
            height,
            frequency,
        } => {
            let mode_info = current_head
                .modes
                .iter()
                .find(|m| {
                    m.width == *width as i32
                        && m.height == *height as i32
                        && if let Some(f) = frequency {
                            let requested = (f * 1000.0) as i32;
                            (m.refresh - requested).abs() < 500
                        } else {
                            true
                        }
                })
                .ok_or(KanskeError::NoConfiguration)?;

            head_config.set_mode(&mode_info.mode);
        }
        OutputCommand::Position { x, y } => {
            head_config.set_position(*x, *y);
        }
        OutputCommand::Scale(scale) => {
            head_config.set_scale(*scale as f64);
        }
        OutputCommand::Transform(transform) => {
            head_config.set_transform(wl_output::Transform::from(*transform))
        }
        OutputCommand::AdaptiveSync(b) => {
            let state = if *b {
                AdaptiveSyncState::Enabled
            } else {
                AdaptiveSyncState::Disabled
            };
            head_config.set_adaptive_sync(state);
        }
        OutputCommand::Enabled(_) => {}
    }
    Ok(())
}
