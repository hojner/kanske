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

    for p in profiles {
        let test = needs_merge(p, outputs.clone());
        dbg!(test);
    }

    // let composed_profiles = profiles
    //     .into_iter()
    //     .for_each(|item| item.outputs.iter().map(|output| output.desc));

    fn needs_merge(profile: &Profile, outputs: Vec<&OutputConfig>) -> bool {
        dbg!(&profile, &outputs);
        profile
            .outputs
            .iter()
            .any(|local| outputs.iter().any(|global| global.desc == local.desc))
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

    todo!();
}
