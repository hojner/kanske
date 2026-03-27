use crate::{
    HeadInfo,
    parser::ast::{Config, ConfigItem, Profile},
};

// pub fn find_matching_profile(heads: &[HeadInfo], config: &Config) -> Profile {
//     for i in &config.items {
//         let matching_heads_len = match i {
//             ConfigItem::Profile(p) => {
//                 match_heads_len(p, heads.len()),
//                 match_heads_names
//             }
//             ConfigItem::Include(_) => todo!(),
//             ConfigItem::Output(_) => todo!(),
//         };
//     }
//     todo!()
// }

// fn match_heads_len(profile: &Profile, no_of_heads: usize) -> bool {
//     if no_of_heads == profile.outputs.len() {
//         return true;
//     }
//     false
// }

// fn match_heads_names(profile: &Profile, )
