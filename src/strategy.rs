pub mod discovery;
pub mod embedded;
pub mod parser;

pub use discovery::get_strategies;
pub use embedded::ensure_custom_strategies;
#[allow(unused_imports)]
pub use parser::ParsedStrategy;
pub use parser::{parse_bat_file, GameFilterPorts};
