use crate::{IndexResource, Resource, ResourceResult};
use futures::future::{err, ok};
use gotham::{
	handler::{HandlerFuture, IntoHandlerError},
	helpers::http::response::create_response,
	pipeline::chain::PipelineHandleChain,
	router::builder::*,
	state::State
};
use mime::APPLICATION_JSON;
use serde::Serialize;
use std::panic::RefUnwindSafe;

pub trait DrawResourceRoutes
{
	fn index<R : Serialize, E : Serialize, Res : ResourceResult<R, E>, IR : IndexResource<R, E, Res>>(&mut self);
}

fn to_handler_future<R, E, Res>(state : State, r : Res) -> Box<HandlerFuture>
where
	R : Serialize,
	E : Serialize,
	Res : ResourceResult<R, E>
{
	let (status, res) = r.to_result();
	let json = match res {
		Ok(json) => serde_json::to_string(&json),
		Err(json) => serde_json::to_string(&json)
	};
	match json {
		Ok(body) => {
			let res = create_response(&state, status, APPLICATION_JSON, body);
			Box::new(ok((state, res)))
		},
		Err(e) => Box::new(err((state, e.into_handler_error())))
	}
}

fn index_handler<R, E, Res, IR>(mut state : State) -> Box<HandlerFuture>
where
	R : Serialize,
	E : Serialize,
	Res : ResourceResult<R, E>,
	IR : IndexResource<R, E, Res>
{
	let res = IR::index(&mut state);
	to_handler_future(state, res)
}

macro_rules! implDrawResourceRoutes {
	($implType:ident) => {
		impl<'a, C, P> DrawResourceRoutes for ($implType<'a, C, P>, String)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			/// Register an `IndexResource` with this resource.
			fn index<R : Serialize, E : Serialize, Res : ResourceResult<R, E>, IR : IndexResource<R, E, Res>>(&mut self)
			{
				self.0.get(&self.1).to(|state| index_handler::<R, E, Res, IR>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);

/// Allows you to setup routes inside a RESTful `Resource`. Currently supported are
/// index (GET without any id), get (GET with an id) and post (POST with a body).
pub struct ResourceSetupRoutes<D : DrawResourceRoutes>
{
	route : D,
	path : String
}

/// This trait adds the `resource` method to gotham's routing. It allows you to register
/// any RESTful `Resource` with a path.
pub trait ResourceRouter
{
	fn resource<R : Resource>(&mut self, path : &str);
}

fn resource<D, C, P, R, T>(route : D, path : T)
where
	C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
	P : RefUnwindSafe + Send + Sync + 'static,
	D : DrawRoutes<C, P>,
	R : Resource,
	T : ToString
{
	R::setup();
}
