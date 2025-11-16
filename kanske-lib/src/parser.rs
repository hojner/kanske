use crate::{AppResult, KanskeError};
pub use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub frequency: f32,
}

#[derive(Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct Scale(f32);

#[derive(Debug)]
pub struct Output {
    pub name: Arc<str>,
    pub enable: bool,
    pub mode: Option<Mode>,
    pub position: Option<Position>,
    pub scale: Option<Scale>,
}

// impl FromStr for Mode {
//     type Err = KanskeError;

//     // "output DP-1 enable mode 3440x1440@60.00Hz position 3,5 scale 1.0"
//     fn from_str(s: &str) -> AppResult<Self> {
//         let mode_str = s.replace("x", " ").replace("@", " ").replace("Hz", " ");
//         let mut mode_parts = mode_str.split_whitespace();
//         let width = mode_parts
//             .next()
//             .and_then(|w| w.parse().ok())
//             .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("width".to_string()))?;

//         let height = mode_parts
//             .next()
//             .and_then(|w| w.parse().ok())
//             .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("width".to_string()))?;

//         let frequency = mode_parts
//             .next()
//             .and_then(|w| w.parse().ok())
//             .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("width".to_string()))?;
//     }
// }

impl FromStr for Output {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mut parts = s.split_whitespace().peekable();
        let name;
        let enable;
        let mode;
        let position;
        let scale;
        if parts.next() == Some(&"output") {
            name = match parts.next() {
                Some(n) => Arc::from(n),
                None => return Err(KanskeError::ParsedStringIsEmpty),
            };
            enable = match parts.next() {
                Some("enable") => true,
                Some("disable") => false,
                Some(e) => return Err(KanskeError::ParsedStringUnexpectedFormat(e.to_string())),
                None => return Err(KanskeError::ParsedStringIsEmpty),
            };
            mode = match parts.next() {
                Some("mode") => {
                    let mode_component = match parts.next() {
                        Some(mc) => {
                            let mode_str =
                                mc.replace("x", ";").replace("@", ";").replace("Hz", ";");
                            let mut mode_parts = mode_str.split(";");
                            let width = match mode_parts.next() {
                                Some(w) => match w.parse() {
                                    Ok(num) => num,
                                    Err(_) => {
                                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                                            "width".to_string(),
                                        ));
                                    }
                                },
                                None => {
                                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                                        "width".to_string(),
                                    ));
                                }
                            };
                            let height = match mode_parts.next() {
                                Some(h) => match h.parse() {
                                    Ok(num) => num,
                                    Err(_) => {
                                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                                            "height".to_string(),
                                        ));
                                    }
                                },
                                None => {
                                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                                        "height".to_string(),
                                    ));
                                }
                            };
                            let frequency = match mode_parts.next() {
                                Some(f) => match f.parse() {
                                    Ok(freq) => freq,
                                    Err(_) => {
                                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                                            "freq".to_string(),
                                        ));
                                    }
                                },
                                None => {
                                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                                        "freq".to_string(),
                                    ));
                                }
                            };
                            Mode {
                                width,
                                height,
                                frequency,
                            }
                        }
                        None => {
                            return Err(KanskeError::ParsedStringUnexpectedFormat(
                                "mode component".to_string(),
                            ));
                        }
                    };
                    Some(mode_component)
                }
                Some(_) => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "mode".to_string(),
                    ));
                }
                None => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "mode".to_string(),
                    ));
                }
            };
            position = match parts.next() {
                Some("position") => {
                    let position_component = match parts.next() {
                        Some(pc) => {
                            let mut pos = pc.split(",");
                            let x = match pos.next() {
                                Some(x) => match x.parse() {
                                    Ok(n) => n,
                                    Err(_e) => {
                                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                                            "position".to_string(),
                                        ));
                                    }
                                },
                                None => {
                                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                                        "postion".to_string(),
                                    ));
                                }
                            };
                            let y = match pos.next() {
                                Some(x) => match x.parse() {
                                    Ok(n) => n,
                                    Err(_e) => {
                                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                                            "position".to_string(),
                                        ));
                                    }
                                },
                                None => {
                                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                                        "postion".to_string(),
                                    ));
                                }
                            };
                            Position { x, y }
                        }
                        None => {
                            return Err(KanskeError::ParsedStringUnexpectedFormat(
                                "postion".to_string(),
                            ));
                        }
                    };
                    Some(position_component)
                }
                Some(_) => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "position".to_string(),
                    ));
                }
                None => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "position".to_string(),
                    ));
                }
            };
            scale = match parts.next() {
                Some("scale") => match parts.next() {
                    Some(sc) => match sc.parse::<f32>() {
                        Ok(n) => Some(Scale(n)),
                        Err(_) => {
                            return Err(KanskeError::ParsedStringUnexpectedFormat(
                                "scale".to_string(),
                            ));
                        }
                    },
                    None => {
                        return Err(KanskeError::ParsedStringUnexpectedFormat(
                            "scale".to_string(),
                        ));
                    }
                },
                Some(_) => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "scale".to_string(),
                    ));
                }
                None => {
                    return Err(KanskeError::ParsedStringUnexpectedFormat(
                        "scale".to_string(),
                    ));
                }
            }
        } else if parts.next() == None {
            return Err(KanskeError::ParsedStringIsEmpty);
        } else {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "output".to_string(),
            ));
        }
        if parts.next() == Some(&"postion") {}
        Ok(Self {
            name,
            enable,
            mode,
            position,
            scale,
        })
    }
}
