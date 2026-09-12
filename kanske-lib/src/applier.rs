use std::collections::HashSet;

use tracing::{debug, error, info};
use wayland_client::{Dispatch, QueueHandle, protocol::wl_output};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
    zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
    zwlr_output_head_v1::AdaptiveSyncState,
};

use crate::{
    AppResult,
    error::KanskeError,
    matcher::find_matching_profile,
    parser::ast::{
        Config, ConfigItem, ExecDirective, OutputCommand, OutputConfig, OutputDesc, Profile,
    },
    wayland_interface::{HeadInfo, WaylandState},
};

/// The name and exec directives of a profile that was just applied. Deliberately does not
/// carry the profile's `outputs`, which are only needed while applying and are otherwise
/// unused by callers (avoids cloning output/command data that's already been consumed).
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedProfile {
    pub name: Option<String>,
    pub execs: Vec<ExecDirective>,
}

/// Finds a matching profile and applies its output configuration.
/// Returns the applied profile (if one matched) and the pending configuration object. The
/// caller must destroy the configuration object after the roundtrip that delivers
/// `Succeeded`/`Failed`/`Cancelled`.
pub fn find_and_apply_profile<D>(
    state: &mut WaylandState,
    qh: &QueueHandle<D>,
    config: &Config,
) -> AppResult<(Option<AppliedProfile>, Option<ZwlrOutputConfigurationV1>)>
where
    D: Dispatch<ZwlrOutputConfigurationV1, ()>
        + Dispatch<ZwlrOutputConfigurationHeadV1, ()>
        + 'static,
{
    match find_matching_profile(&state.heads, config) {
        Some(profile) => apply_profile(state, qh, profile),
        None => Ok((None, None)),
    }
}

/// Looks up a profile by name in `config` and applies it regardless of whether it matches
/// the currently connected heads. Used for manual `kanskectl switch <profile>` requests.
/// Returns `KanskeError::ProfileNotFound` if no profile with that name exists.
pub fn apply_named_profile<D>(
    state: &mut WaylandState,
    qh: &QueueHandle<D>,
    config: &Config,
    name: &str,
) -> AppResult<(Option<AppliedProfile>, Option<ZwlrOutputConfigurationV1>)>
where
    D: Dispatch<ZwlrOutputConfigurationV1, ()>
        + Dispatch<ZwlrOutputConfigurationHeadV1, ()>
        + 'static,
{
    let profile = config
        .items
        .iter()
        .find_map(|item| match item {
            ConfigItem::Profile(p) if p.name.as_deref() == Some(name) => Some(p),
            _ => None,
        })
        .ok_or_else(|| KanskeError::ProfileNotFound {
            name: name.to_string(),
        })?;
    apply_profile(state, qh, profile)
}

/// Applies the given profile's output configuration to the currently connected heads.
/// Returns the applied profile and the pending configuration object. The caller must
/// destroy the configuration object after the roundtrip that delivers
/// `Succeeded`/`Failed`/`Cancelled`.
fn apply_profile<D>(
    state: &mut WaylandState,
    qh: &QueueHandle<D>,
    profile: &Profile,
) -> AppResult<(Option<AppliedProfile>, Option<ZwlrOutputConfigurationV1>)>
where
    D: Dispatch<ZwlrOutputConfigurationV1, ()>
        + Dispatch<ZwlrOutputConfigurationHeadV1, ()>
        + 'static,
{
    info!(profile = ?profile.name, "Applying profile");
    let mut used_indices: HashSet<usize> = HashSet::new();
    let manager = state
        .manager
        .as_ref()
        .ok_or(KanskeError::ManagerNotAvailable)?;
    let serial = state.serial.ok_or(KanskeError::NoSerial)?;
    let output_configuration = manager.create_configuration(serial, qh, ());

    if let Err(e) = configure_outputs(state, qh, profile, &output_configuration, &mut used_indices)
    {
        // Some heads may have already been staged on this configuration object before
        // the failure (e.g. a named output with no matching head). Since `.apply()` is
        // never reached, the object would otherwise never be applied nor destroyed —
        // leaking it both client- and server-side.
        output_configuration.destroy();
        return Err(e);
    }

    output_configuration.apply();
    let applied = AppliedProfile {
        name: profile.name.clone(),
        execs: profile.execs.clone(),
    };
    Ok((Some(applied), Some(output_configuration)))
}

/// Stages `enable_head`/`disable_head` requests (and their commands) on `output_configuration`
/// for every output in `profile`, matching named/described outputs first and wildcards last.
/// Does not call `.apply()` or `.destroy()` — that's the caller's responsibility.
fn configure_outputs<D>(
    state: &WaylandState,
    qh: &QueueHandle<D>,
    profile: &Profile,
    output_configuration: &ZwlrOutputConfigurationV1,
    used_indices: &mut HashSet<usize>,
) -> AppResult<()>
where
    D: Dispatch<ZwlrOutputConfigurationV1, ()>
        + Dispatch<ZwlrOutputConfigurationHeadV1, ()>
        + 'static,
{
    for output in profile
        .outputs
        .iter()
        .filter(|f| matches!(f.desc, OutputDesc::Name(_) | OutputDesc::Description(_)))
    {
        let position = state
            .heads
            .iter()
            .enumerate()
            .find(|(i, h)| !used_indices.contains(i) && output.desc.matches(h))
            .map(|(i, _)| i)
            .ok_or_else(|| {
                let name = match &output.desc {
                    OutputDesc::Name(n) => n.clone(),
                    OutputDesc::Description(d) => d.clone(),
                    OutputDesc::Any => "*".to_string(),
                };
                KanskeError::HeadNotFound { name }
            })?;
        used_indices.insert(position);
        let current_head = &state.heads[position];
        debug!(output = ?output.desc, head = %current_head.name, "Named output matched to head");
        configure_head(output, output_configuration, current_head, qh)?;
    }
    for output in profile
        .outputs
        .iter()
        .filter(|f| matches!(f.desc, OutputDesc::Any))
    {
        let position = (0..state.heads.len())
            .find(|i| !used_indices.contains(i))
            .ok_or_else(|| KanskeError::HeadNotFound {
                name: "*".to_string(),
            })?;
        used_indices.insert(position);
        let current_head = &state.heads[position];
        debug!(head = %current_head.name, "Wildcard output consuming head");
        configure_head(output, output_configuration, current_head, qh)?;
    }
    Ok(())
}

fn configure_head<D>(
    output: &OutputConfig,
    output_configuration: &ZwlrOutputConfigurationV1,
    current_head: &HeadInfo,
    qh: &QueueHandle<D>,
) -> AppResult<()>
where
    D: Dispatch<ZwlrOutputConfigurationV1, ()>
        + Dispatch<ZwlrOutputConfigurationHeadV1, ()>
        + 'static,
{
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
        debug!(head = %current_head.name, commands = output.commands.len(), "Enabling head");
        let head_config = output_configuration.enable_head(&current_head.head, qh, ());
        for command in output.commands.iter() {
            apply_command(&head_config, command, current_head)?;
        }
    } else {
        debug!(head = %current_head.name, "Disabling head");
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
            let mode_info = current_head.modes.iter().find(|m| {
                m.width == *width as i32
                    && m.height == *height as i32
                    && if let Some(f) = frequency {
                        let requested = (f * 1000.0) as i32;
                        (m.refresh - requested).abs() < 500
                    } else {
                        true
                    }
            });

            match mode_info {
                Some(m) => {
                    debug!(
                        head = %current_head.name,
                        width = m.width,
                        height = m.height,
                        refresh = m.refresh,
                        "Setting mode"
                    );
                    head_config.set_mode(&m.mode);
                }
                None => {
                    error!(
                        head = %current_head.name,
                        requested_width = width,
                        requested_height = height,
                        requested_freq = ?frequency,
                        available_modes = current_head.modes.len(),
                        "Requested mode not found, skipping"
                    );
                }
            }
        }
        OutputCommand::Position { x, y } => {
            debug!(head = %current_head.name, x, y, "Setting position");
            head_config.set_position(*x, *y);
        }
        OutputCommand::Scale(scale) => {
            debug!(head = %current_head.name, scale, "Setting scale");
            head_config.set_scale(*scale as f64);
        }
        OutputCommand::Transform(transform) => {
            debug!(head = %current_head.name, transform = ?transform, "Setting transform");
            head_config.set_transform(wl_output::Transform::from(*transform))
        }
        OutputCommand::AdaptiveSync(b) => {
            debug!(head = %current_head.name, adaptive_sync = b, "Setting adaptive sync");
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
