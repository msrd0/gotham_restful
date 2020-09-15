use crate::{DrawResourceRoutes, RequestBody, ResourceID, ResourceResult, ResourceType};
use gotham::{extractor::QueryStringExtractor, hyper::Body, state::State};
use std::{future::Future, pin::Pin};

/// This trait must be implemented for every resource. It allows you to register the different
/// methods that can be handled by this resource to be registered with the underlying router.
///
/// It is not recommended to implement this yourself, rather just use `#[derive(Resource)]`.
pub trait Resource {
	/// Register all methods handled by this resource with the underlying router.
	fn setup<D: DrawResourceRoutes>(route: D);
}

/// A common trait for every resource method. It defines the return type as well as some general
/// information about a resource method.
///
/// It is not recommended to implement this yourself. Rather, just write your handler method and
/// annotate it with `#[<method>(YourResource)]`, where `<method>` is one of the supported
/// resource methods.
pub trait ResourceMethod {
	type Res: ResourceResult + Send + 'static;

	#[cfg(feature = "openapi")]
	fn operation_id() -> Option<String> {
		None
	}

	fn wants_auth() -> bool {
		false
	}
}

/// The read_all [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceReadAll: ResourceMethod {
	/// Handle a GET request on the Resource root.
	fn read_all(state: State) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The read [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceRead: ResourceMethod {
	/// The ID type to be parsed from the request path.
	type ID: ResourceID + 'static;

	/// Handle a GET request on the Resource with an id.
	fn read(state: State, id: Self::ID) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The search [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceSearch: ResourceMethod {
	/// The Query type to be parsed from the request parameters.
	type Query: ResourceType + QueryStringExtractor<Body> + Sync;

	/// Handle a GET request on the Resource with additional search parameters.
	fn search(state: State, query: Self::Query) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The create [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceCreate: ResourceMethod {
	/// The Body type to be parsed from the request body.
	type Body: RequestBody;

	/// Handle a POST request on the Resource root.
	fn create(state: State, body: Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The change_all [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceChangeAll: ResourceMethod {
	/// The Body type to be parsed from the request body.
	type Body: RequestBody;

	/// Handle a PUT request on the Resource root.
	fn change_all(state: State, body: Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The change [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceChange: ResourceMethod {
	/// The Body type to be parsed from the request body.
	type Body: RequestBody;
	/// The ID type to be parsed from the request path.
	type ID: ResourceID + 'static;

	/// Handle a PUT request on the Resource with an id.
	fn change(state: State, id: Self::ID, body: Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The remove_all [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceRemoveAll: ResourceMethod {
	/// Handle a DELETE request on the Resource root.
	fn remove_all(state: State) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// The remove [`ResourceMethod`](trait.ResourceMethod.html).
pub trait ResourceRemove: ResourceMethod {
	/// The ID type to be parsed from the request path.
	type ID: ResourceID + 'static;

	/// Handle a DELETE request on the Resource with an id.
	fn remove(state: State, id: Self::ID) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}
