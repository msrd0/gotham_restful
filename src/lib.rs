#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate serde;

pub use hyper::StatusCode;

mod resource;
pub use resource::{
	Resource,
	IndexResource,
	GetResource,
	CreateResource,
	ChangeAllResource,
	ChangeResource
};

mod result;
pub use result::{ResourceResult, Success};

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
