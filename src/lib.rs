#[macro_use]
extern crate serde_derive;

pub use hyper::StatusCode;

mod resource;
pub use resource::{
	Resource,
	IndexResource,
	GetResource,
	PostResource
};

mod result;
pub use result::ResourceResult;

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
