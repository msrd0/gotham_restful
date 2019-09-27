use crate::{GetResource, IndexResource, Resource, ResourceResult};
use futures::future::{err, ok};
use gotham::{
	handler::{HandlerFuture, IntoHandlerError},
	helpers::http::response::create_response,
	pipeline::chain::PipelineHandleChain,
	router::builder::*,
	state::{FromState, State}
};
use mime::APPLICATION_JSON;
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

/// Allow us to extract an id from a path.
#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathExtractor<ID : RefUnwindSafe + Send + 'static>
{
	id : ID
}

/// This trait adds the `resource` method to gotham's routing. It allows you to register
/// any RESTful `Resource` with a path.
pub trait DrawResources
{
	fn resource<R : Resource, T : ToString>(&mut self, path : T);
}

/// This trait allows to draw routes within an resource. Use this only inside the
/// `Resource::setup` method.
pub trait DrawResourceRoutes
{
	fn index<R, IR>(&mut self)
	where
		R : ResourceResult,
		IR : IndexResource<R>;
	
	fn get<ID, R, IR>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		R : ResourceResult,
		IR : GetResource<ID, R>;
}

fn to_handler_future<F, R>(mut state : State, get_result : F) -> Box<HandlerFuture>
where
	F : FnOnce(&mut State) -> R,
	R : ResourceResult
{
	let res = get_result(&mut state).to_json();
	match res {
		Ok((status, body)) => {
			let res = create_response(&state, status, APPLICATION_JSON, body);
			Box::new(ok((state, res)))
		},
		Err(e) => Box::new(err((state, e.into_handler_error())))
	}
}

fn index_handler<R : ResourceResult, IR : IndexResource<R>>(state : State) -> Box<HandlerFuture>
{
	to_handler_future(state, |state| IR::index(state))
}

fn get_handler<ID, R : ResourceResult, GR : GetResource<ID, R>>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	to_handler_future(state, |state| GR::get(state, id))
}

macro_rules! implDrawResourceRoutes {
	($implType:ident) => {
		impl<'a, C, P> DrawResources for $implType<'a, C, P>
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R : Resource, T : ToString>(&mut self, path : T)
			{
				R::setup((self, path.to_string()));
			}
		}
		
		impl<'a, C, P> DrawResourceRoutes for (&mut $implType<'a, C, P>, String)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn index<R, IR>(&mut self)
			where
				R : ResourceResult,
				IR : IndexResource<R>
			{
				self.0.get(&self.1)
					.to(|state| index_handler::<R, IR>(state));
			}

			fn get<ID, R, IR>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				R : ResourceResult,
				IR : GetResource<ID, R>
			{
				self.0.get(&format!("{}/:id", self.1))
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| get_handler::<ID, R, IR>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
