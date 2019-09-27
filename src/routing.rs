use crate::{
	result::ResourceError,
	ChangeAllResource,
	ChangeResource,
	CreateResource,
	GetResource,
	IndexResource,
	Resource,
	ResourceResult,
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
	fn index<R, IR>(&mut self)
	where
		R : ResourceResult,
		IR : IndexResource<R>;
	
	fn get<ID, R, GR>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		R : ResourceResult,
		GR : GetResource<ID, R>;
	
	fn create<Body, R, CR>(&mut self)
	where
		Body : DeserializeOwned,
		R : ResourceResult,
		CR : CreateResource<Body, R>;

	fn change_all<Body, R, CR>(&mut self)
	where
		Body : DeserializeOwned,
		R : ResourceResult,
		CR : ChangeAllResource<Body, R>;

	fn change<ID, Body, R, CR>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		Body : DeserializeOwned,
		R : ResourceResult,
		CR : ChangeResource<ID, Body, R>;
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

fn create_handler<Body : DeserializeOwned, R : ResourceResult, CR : CreateResource<Body, R>>(state : State) -> Box<HandlerFuture>
{
	handle_with_body::<Body, _, _>(state, |state, body| CR::create(state, body))
}

fn change_all_handler<Body : DeserializeOwned, R : ResourceResult, CR : ChangeAllResource<Body, R>>(state : State) -> Box<HandlerFuture>
{
	handle_with_body::<Body, _, _>(state, |state, body| CR::change_all(state, body))
}

fn change_handler<ID, Body, R, CR>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
	Body : DeserializeOwned,
	R : ResourceResult,
	CR : ChangeResource<ID, Body, R>
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	handle_with_body::<Body, _, _>(state, |state, body| CR::change(state, id, body))
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

			fn create<Body, R, CR>(&mut self)
			where
				Body : DeserializeOwned,
				R : ResourceResult,
				CR : CreateResource<Body, R>
			{
				self.0.post(&self.1)
					.to(|state| create_handler::<Body, R, CR>(state));
			}

			fn change_all<Body, R, CR>(&mut self)
			where
				Body : DeserializeOwned,
				R : ResourceResult,
				CR : ChangeAllResource<Body, R>
			{
				self.0.put(&self.1)
					.to(|state| change_all_handler::<Body, R, CR>(state));
			}

			fn change<ID, Body, R, CR>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Body : DeserializeOwned,
				R : ResourceResult,
				CR : ChangeResource<ID, Body, R>
			{
				self.0.put(&format!("{}/:id", self.1))
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| change_handler::<ID, Body, R, CR>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
