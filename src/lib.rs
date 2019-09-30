#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate serde;

pub use hyper::StatusCode;

pub mod helper;

mod resource;
pub use resource::{
	Resource,
	ResourceReadAll,
	ResourceRead,
	ResourceCreate,
	ResourceUpdateAll,
	ResourceUpdate,
	ResourceDeleteAll,
	ResourceDelete
};

mod result;
pub use result::{ResourceResult, Success};

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
