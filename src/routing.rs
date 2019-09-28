use crate::{
	resource::*,
	result::{ResourceError, ResourceResult},
	StatusCode
};
use futures::{
	future::{Future, err, ok},
	stream::Stream
};
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
	fn read_all<Handler, Res>(&mut self)
	where
		Res : ResourceResult,
		Handler : ResourceReadAll<Res>;
	
	fn read<Handler, ID, Res>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		Res : ResourceResult,
		Handler : ResourceRead<ID, Res>;
	
	fn create<Handler, Body, Res>(&mut self)
	where
		Body : DeserializeOwned,
		Res : ResourceResult,
		Handler : ResourceCreate<Body, Res>;

	fn update_all<Handler, Body, Res>(&mut self)
	where
		Body : DeserializeOwned,
		Res : ResourceResult,
		Handler : ResourceUpdateAll<Body, Res>;

	fn update<Handler, ID, Body, Res>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		Body : DeserializeOwned,
		Res : ResourceResult,
		Handler : ResourceUpdate<ID, Body, Res>;
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

fn handle_with_body<Body, F, R>(mut state : State, get_result : F) -> Box<HandlerFuture>
where
	Body : DeserializeOwned,
	F : FnOnce(&mut State, Body) -> R + Send + 'static,
	R : ResourceResult
{
	let f = hyper::Body::take_from(&mut state)
		.concat2()
		.then(|body| {

			let body = match body {
				Ok(body) => body,
				Err(e) => return err((state, e.into_handler_error()))
			};

			let body = match serde_json::from_slice(&body) {
				Ok(body) => body,
				Err(e) => return {
					let error : ResourceError = e.into();
					match serde_json::to_string(&error) {
						Ok(json) => {
							let res = create_response(&state, StatusCode::BAD_REQUEST, APPLICATION_JSON, json);
							ok((state, res))
						},
						Err(e) => err((state, e.into_handler_error()))
					}
				}
			};

			let res = get_result(&mut state, body).to_json();
			match res {
				Ok((status, body)) => {
					let res = create_response(&state, status, APPLICATION_JSON, body);
					ok((state, res))
				},
				Err(e) => err((state, e.into_handler_error()))
			}
			
		});

	Box::new(f)
}

fn read_all_handler<Handler, Res>(state : State) -> Box<HandlerFuture>
where
	Res : ResourceResult,
	Handler : ResourceReadAll<Res>
{
	to_handler_future(state, |state| Handler::read_all(state))
}

fn read_handler<Handler, ID, Res>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
	Res : ResourceResult,
	Handler : ResourceRead<ID, Res>
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	to_handler_future(state, |state| Handler::read(state, id))
}

fn create_handler<Handler, Body, Res>(state : State) -> Box<HandlerFuture>
where
	Body : DeserializeOwned,
	Res : ResourceResult,
	Handler : ResourceCreate<Body, Res>
{
	handle_with_body::<Body, _, _>(state, |state, body| Handler::create(state, body))
}

fn update_all_handler<Handler, Body, Res>(state : State) -> Box<HandlerFuture>
where
	Body : DeserializeOwned,
	Res : ResourceResult,
	Handler : ResourceUpdateAll<Body, Res>
{
	handle_with_body::<Body, _, _>(state, |state, body| Handler::update_all(state, body))
}

fn update_handler<Handler, ID, Body, Res>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
	Body : DeserializeOwned,
	Res : ResourceResult,
	Handler : ResourceUpdate<ID, Body, Res>
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	handle_with_body::<Body, _, _>(state, |state, body| Handler::update(state, id, body))
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
			fn read_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceReadAll<Res>
			{
				self.0.get(&self.1)
					.to(|state| read_all_handler::<Handler, Res>(state));
			}

			fn read<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceRead<ID, Res>
			{
				self.0.get(&format!("{}/:id", self.1))
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| read_handler::<Handler, ID, Res>(state));
			}

			fn create<Handler, Body, Res>(&mut self)
			where
				Body : DeserializeOwned,
				Res : ResourceResult,
				Handler : ResourceCreate<Body, Res>
			{
				self.0.post(&self.1)
					.to(|state| create_handler::<Handler, Body, Res>(state));
			}

			fn update_all<Handler, Body, Res>(&mut self)
			where
				Body : DeserializeOwned,
				Res : ResourceResult,
				Handler : ResourceUpdateAll<Body, Res>
			{
				self.0.put(&self.1)
					.to(|state| update_all_handler::<Handler, Body, Res>(state));
			}

			fn update<Handler, ID, Body, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Body : DeserializeOwned,
				Res : ResourceResult,
				Handler : ResourceUpdate<ID, Body, Res>
			{
				self.0.put(&format!("{}/:id", self.1))
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| update_handler::<Handler, ID, Body, Res>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
