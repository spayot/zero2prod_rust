pub mod dashboard;
pub mod password;
pub mod logout;
pub mod newsletters;

pub use dashboard::admin_dashboard;
pub use password::*;
pub use logout::log_out;
pub use newsletters::*;