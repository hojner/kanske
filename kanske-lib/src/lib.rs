pub mod composer;
pub mod error;
pub mod matcher;
pub mod parser;
pub mod wayland_interface;

pub use error::AppResult;

use crate::wayland_interface::HeadInfo;
use wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1;

#[derive(Debug)]
pub struct KanskeState {
    pub manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    pub heads: Vec<HeadInfo>,
    pub serial: Option<u32>,
}
