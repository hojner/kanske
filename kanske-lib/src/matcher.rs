use tracing::{debug, trace};

use crate::{
    parser::ast::{Config, ConfigItem, OutputDesc, Profile},
    wayland_interface::HeadInfo,
};

pub fn find_matching_profile<'a>(heads: &[HeadInfo], config: &'a Config) -> Option<&'a Profile> {
    let head_names: Vec<&str> = heads.iter().map(|h| h.name.as_str()).collect();
    debug!(heads = ?head_names, "Finding matching profile");

    let result = config.items.iter().find_map(|i| {
        if let ConfigItem::Profile(p) = i {
            if profile_matches(heads, p) {
                Some(p)
            } else {
                None
            }
        } else {
            None
        }
    });

    match &result {
        Some(p) => debug!(profile = ?p.name, "Profile matched"),
        None => debug!("No profile matched"),
    }
    result
}

fn profile_matches(heads: &[HeadInfo], profile: &Profile) -> bool {
    let profile_name = profile.name.as_deref().unwrap_or("<anonymous>");

    if heads.len() != profile.outputs.len() {
        trace!(
            profile = profile_name,
            head_count = heads.len(),
            output_count = profile.outputs.len(),
            "Rejected: head count mismatch"
        );
        return false;
    }

    let mut used = vec![false; heads.len()];

    // Named outputs must each match a unique head
    for output in profile.outputs.iter() {
        if let OutputDesc::Name(name) = &output.desc {
            let found = heads
                .iter()
                .enumerate()
                .find(|(i, h)| !used[*i] && h.name == *name);
            match found {
                Some((i, _)) => used[i] = true,
                None => {
                    trace!(profile = profile_name, output = %name, "Rejected: no head for named output");
                    return false;
                }
            }
        }
    }

    // Wildcards consume remaining unmatched heads
    let remaining = used.iter().filter(|u| !**u).count();
    let wildcards = profile
        .outputs
        .iter()
        .filter(|o| matches!(o.desc, OutputDesc::Any))
        .count();

    remaining == wildcards
}
