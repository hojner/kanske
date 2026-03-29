use crate::{
    parser::ast::{Config, ConfigItem, Profile},
    wayland_interface::HeadInfo,
};

pub fn find_matching_profile<'a>(heads: &[HeadInfo], config: &'a Config) -> Option<&'a Profile> {
    config.items.iter().find_map(|i| {
        if let ConfigItem::Profile(p) = i {
            let matches = heads.len() == p.outputs.len()
                && p.outputs
                    .iter()
                    .all(|po| heads.iter().any(|h| po.desc.matches(&h.name)));
            if matches { Some(p) } else { None }
        } else {
            None
        }
    })
}
