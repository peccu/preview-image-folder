mod page;
pub use crate::page::genpage;

mod app;
pub use crate::app::AppParam;

pub mod files;

mod server;
pub use crate::server::spawn_server;
