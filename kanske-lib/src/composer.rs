use std::mem::discriminant;

use crate::parser::ast::{
    Config, ConfigItem, IncludeDirective, OutputCommand, OutputConfig, Profile,
};

pub fn compose_profiles(config: Config) -> Config {
    let (profiles, outputs, includes): (Vec<&Profile>, Vec<&OutputConfig>, Vec<&IncludeDirective>) =
        config.items.iter().fold(
            (vec![], vec![], vec![]),
            |(mut profiles, mut outputs, mut includes), item| {
                match item {
                    ConfigItem::Profile(p) => profiles.push(p),
                    ConfigItem::Output(o) => outputs.push(o),
                    ConfigItem::Include(i) => includes.push(i),
                }
                (profiles, outputs, includes)
            },
        );
    if outputs.is_empty() {
        return config;
    }

    let mut composed: Vec<ConfigItem> = profiles
        .into_iter()
        .map(|p| {
            let merged_outputs: Vec<OutputConfig> = p
                .outputs
                .iter()
                .map(
                    |local| match outputs.iter().find(|global| global.desc == local.desc) {
                        Some(global) => merge_output(global, local),
                        None => local.clone(),
                    },
                )
                .collect();
            ConfigItem::Profile(Profile {
                name: p.name.clone(),
                outputs: merged_outputs,
                execs: p.execs.clone(),
            })
        })
        .collect();

    composed.extend(includes.into_iter().map(|i| ConfigItem::Include(i.clone())));

    Config { items: composed }
}

fn merge_output(global_output: &OutputConfig, local_output: &OutputConfig) -> OutputConfig {
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
    OutputConfig {
        desc: local_output.desc.clone(),
        commands: result,
    }
}
