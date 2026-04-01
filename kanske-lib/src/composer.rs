use std::mem::discriminant;

use crate::{
    AppResult,
    parser::ast::{Config, ConfigItem, IncludeDirective, OutputCommand, OutputConfig, Profile},
};

pub fn compose_profiles(config: Config) -> AppResult<Config> {
    let mut profiles: Vec<Profile> = Vec::new();
    let mut outputs: Vec<OutputConfig> = Vec::new();
    let mut includes: Vec<IncludeDirective> = Vec::new();

    for item in config.items.into_iter() {
        match item {
            ConfigItem::Profile(p) => profiles.push(p),
            ConfigItem::Output(o) => outputs.push(o),
            ConfigItem::Include(i) => includes.push(i),
        }
    }
    if outputs.is_empty() {
        let items = profiles
            .into_iter()
            .map(ConfigItem::Profile)
            .chain(includes.into_iter().map(ConfigItem::Include))
            .collect();
        return Ok(Config { items });
    }

    let mut composed: Vec<ConfigItem> = profiles
        .into_iter()
        .map(|p| -> AppResult<ConfigItem> {
            let merged_outputs: Vec<OutputConfig> = p
                .outputs
                .iter()
                .map(
                    |local| match outputs.iter().find(|global| global.desc == local.desc) {
                        Some(global) => merge_output(global, local),

                        None => Ok(local.clone()),
                    },
                )
                .collect::<AppResult<Vec<_>>>()?;
            Ok(ConfigItem::Profile(Profile {
                name: p.name,
                outputs: merged_outputs,
                execs: p.execs,
            }))
        })
        .collect::<AppResult<Vec<_>>>()?;

    composed.extend(includes.into_iter().map(|i| ConfigItem::Include(i)));

    Ok(Config { items: composed })
}

fn merge_output(
    global_output: &OutputConfig,
    local_output: &OutputConfig,
) -> AppResult<OutputConfig> {
    let mut result: Vec<OutputCommand> = global_output
        .commands
        .iter()
        .filter(|g| {
            !local_output
                .commands
                .iter()
                .any(|l| discriminant(*g) == discriminant(l))
        })
        .cloned()
        .collect();
    result.extend(local_output.commands.iter().cloned());
    Ok(OutputConfig {
        desc: local_output.desc.clone(),
        commands: result,
    })
}
