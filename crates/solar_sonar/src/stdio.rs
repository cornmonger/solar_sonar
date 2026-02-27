use crate::*;

pub(crate) const INFO: &'static str = formatcp!("{CYAN}[sonar]{CLR}");
pub(crate) const ERR: &'static str = formatcp!("{RED}[sonar]{CLR}");

pub(crate) const CLR: &'static str = "\x1b[0m";
pub(crate) const WHITE: &'static str = "\x1b[37m";
pub(crate) const RED: &'static str = "\x1b[31m";
pub(crate) const ORANGE: &'static str = "\x1b[38;2;139;69;0m";
pub(crate) const WHITE_ON_RED: &'static str = "\x1b[41;37m";
pub(crate) const WHITE_ON_ORANGE: &'static str = "\x1b[37;48;2;139;69;0m";
pub(crate) const GREEN: &'static str = "\x1b[32m";
pub(crate) const YELLOW: &'static str = "\x1b[33m";
pub(crate) const BRIGHT_BLUE: &'static str = "\x1b[94m";
pub(crate) const MAGENTA: &'static str = "\x1b[35m";
pub(crate) const CYAN: &'static str = "\x1b[36m";
pub(crate) const GRAY: &'static str = "\x1b[90m";
