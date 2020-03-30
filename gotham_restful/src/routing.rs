use crate::{
	resource::*,
	result::{ResourceError, ResourceResult, Response},
	RequestBody,
	ResourceType,
	StatusCode
};
#[cfg(feature = "openapi")]
use crate::OpenapiRouter;

use futures::{
	future::{Future, err, ok},
	stream::Stream
};
use gotham::{
	extractor::QueryStringExtractor,
	handler::{HandlerFuture, IntoHandlerError},
	helpers::http::response::{create_empty_response, create_response},
	pipeline::chain::PipelineHandleChain,
	router::{
		builder::*,
		non_match::RouteNonMatch,
		route::matcher::{
			content_type::ContentTypeHeaderRouteMatcher,
			AcceptHeaderRouteMatcher,
			RouteMatcher
		}
	},
	state::{FromState, State}
};
use hyper::{
	header::CONTENT_TYPE,
	Body,
	HeaderMap,
	Method
};
use mime::{Mime, APPLICATION_JSON};
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

/// Allow us to extract an id from a path.
#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathExtractor<ID : RefUnwindSafe + Send + 'static>
{
	id : ID
}

/// This trait adds the `with_openapi` method to gotham's routing. It turns the default
/// router into one that will only allow RESTful resources, but record them and generate
/// an OpenAPI specification on request.
#[cfg(feature = "openapi")]
pub trait WithOpenapi<D>
{
	fn with_openapi<F>(&mut self, title : String, version : String, server_url : String, block : F)
	where
		F : FnOnce((&mut D, &mut OpenapiRouter));
}

/// This trait adds the `resource` method to gotham's routing. It allows you to register
/// any RESTful `Resource` with a path.
pub trait DrawResources
{
	fn resource<R : Resource>(&mut self, path : &str);
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
	
	fn search<Handler, Query, Res>(&mut self)
	where
		Query : ResourceType + DeserializeOwned + QueryStringExtractor<Body> + Send + Sync + 'static,
		Res : ResourceResult,
		Handler : ResourceSearch<Query, Res>;
	
	fn create<Handler, Body, Res>(&mut self)
	where
		Body : RequestBody,
		Res : ResourceResult,
		Handler : ResourceCreate<Body, Res>;

	fn update_all<Handler, Body, Res>(&mut self)
	where
		Body : RequestBody,
		Res : ResourceResult,
		Handler : ResourceUpdateAll<Body, Res>;

	fn update<Handler, ID, Body, Res>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		Body : RequestBody,
		Res : ResourceResult,
		Handler : ResourceUpdate<ID, Body, Res>;

	fn delete_all<Handler, Res>(&mut self)
	where
		Res : ResourceResult,
		Handler : ResourceDeleteAll<Res>;
	
	fn delete<Handler, ID, Res>(&mut self)
	where
		ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
		Res : ResourceResult,
		Handler : ResourceDelete<ID, Res>;
}

fn response_from(res : Response, state : &State) -> hyper::Response<Body>
{
	let mut r = create_empty_response(state, res.status);
	if let Some(mime) = res.mime
	{
		r.headers_mut().insert(CONTENT_TYPE, mime.as_ref().parse().unwrap());
	}
	if Method::borrow_from(state) != Method::HEAD
	{
		*r.body_mut() = res.body;
	}
	r
}

fn to_handler_future<F, R>(mut state : State, get_result : F) -> Box<HandlerFuture>
where
	F : FnOnce(&mut State) -> R,
	R : ResourceResult
{
	let res = get_result(&mut state).into_response();
	match res {
		Ok(res) => {
			let r = response_from(res, &state);
			Box::new(ok((state, r)))
		},
		Err(e) => Box::new(err((state, e.into_handler_error())))
	}
}

fn handle_with_body<Body, F, R>(mut state : State, get_result : F) -> Box<HandlerFuture>
where
	Body : RequestBody,
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
			
			let content_type : Mime = match HeaderMap::borrow_from(&state).get(CONTENT_TYPE) {
				Some(content_type) => content_type.to_str().unwrap().parse().unwrap(),
				None => {
					let res = create_empty_response(&state, StatusCode::UNSUPPORTED_MEDIA_TYPE);
					return ok((state, res))
				}
			};

			let body = match Body::from_body(body, content_type) {
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

			let res = get_result(&mut state, body).into_response();
			match res {
				Ok(res) => {
					let r = response_from(res, &state);
					ok((state, r))
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

fn search_handler<Handler, Query, Res>(mut state : State) -> Box<HandlerFuture>
where
	Query : ResourceType + QueryStringExtractor<Body> + Send + Sync + 'static,
	Res : ResourceResult,
	Handler : ResourceSearch<Query, Res>
{
	let query = Query::take_from(&mut state);
	to_handler_future(state, |state| Handler::search(state, query))
}

fn create_handler<Handler, Body, Res>(state : State) -> Box<HandlerFuture>
where
	Body : RequestBody,
	Res : ResourceResult,
	Handler : ResourceCreate<Body, Res>
{
	handle_with_body::<Body, _, _>(state, |state, body| Handler::create(state, body))
}

fn update_all_handler<Handler, Body, Res>(state : State) -> Box<HandlerFuture>
where
	Body : RequestBody,
	Res : ResourceResult,
	Handler : ResourceUpdateAll<Body, Res>
{
	handle_with_body::<Body, _, _>(state, |state, body| Handler::update_all(state, body))
}

fn update_handler<Handler, ID, Body, Res>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
	Body : RequestBody,
	Res : ResourceResult,
	Handler : ResourceUpdate<ID, Body, Res>
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	handle_with_body::<Body, _, _>(state, |state, body| Handler::update(state, id, body))
}

fn delete_all_handler<Handler, Res>(state : State) -> Box<HandlerFuture>
where
	Res : ResourceResult,
	Handler : ResourceDeleteAll<Res>
{
	to_handler_future(state, |state| Handler::delete_all(state))
}

fn delete_handler<Handler, ID, Res>(state : State) -> Box<HandlerFuture>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
	Res : ResourceResult,
	Handler : ResourceDelete<ID, Res>
{
	let id = {
		let path : &PathExtractor<ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	to_handler_future(state, |state| Handler::delete(state, id))
}

#[derive(Clone)]
struct MaybeMatchAcceptHeader
{
	matcher : Option<AcceptHeaderRouteMatcher>
}

impl RouteMatcher for MaybeMatchAcceptHeader
{
	fn is_match(&self, state : &State) -> Result<(), RouteNonMatch>
	{
		match &self.matcher {
			Some(matcher) => matcher.is_match(state),
			None => Ok(())
		}
	}
}

impl From<Option<Vec<Mime>>> for MaybeMatchAcceptHeader
{
	fn from(types : Option<Vec<Mime>>) -> Self
	{
		Self {
			matcher: types.map(AcceptHeaderRouteMatcher::new)
		}
	}
}

#[derive(Clone)]
struct MaybeMatchContentTypeHeader
{
	matcher : Option<ContentTypeHeaderRouteMatcher>
}

impl RouteMatcher for MaybeMatchContentTypeHeader
{
	fn is_match(&self, state : &State) -> Result<(), RouteNonMatch>
	{
		match &self.matcher {
			Some(matcher) => matcher.is_match(state),
			None => Ok(())
		}
	}
}

impl From<Option<Vec<Mime>>> for MaybeMatchContentTypeHeader
{
	fn from(types : Option<Vec<Mime>>) -> Self
	{
		Self {
			matcher: types.map(ContentTypeHeaderRouteMatcher::new)
		}
	}
}

macro_rules! implDrawResourceRoutes {
	($implType:ident) => {
		
		#[cfg(feature = "openapi")]
		impl<'a, C, P> WithOpenapi<Self> for $implType<'a, C, P>
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn with_openapi<F>(&mut self, title : String, version : String, server_url : String, block : F)
			where
				F : FnOnce((&mut Self, &mut OpenapiRouter))
			{
				let mut router = OpenapiRouter::new(title, version, server_url);
				block((self, &mut router));
			}
		}
		
		impl<'a, C, P> DrawResources for $implType<'a, C, P>
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R : Resource>(&mut self, path : &str)
			{
				R::setup((self, path));
			}
		}
		
		#[allow(clippy::redundant_closure)] // doesn't work because of type parameters
		impl<'a, C, P> DrawResourceRoutes for (&mut $implType<'a, C, P>, &str)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn read_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceReadAll<Res>
			{
				let matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				self.0.get(&self.1)
					.extend_route_matcher(matcher)
					.to(|state| read_all_handler::<Handler, Res>(state));
			}

			fn read<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceRead<ID, Res>
			{
				let matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				self.0.get(&format!("{}/:id", self.1))
					.extend_route_matcher(matcher)
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| read_handler::<Handler, ID, Res>(state));
			}
			
			fn search<Handler, Query, Res>(&mut self)
			where
				Query : ResourceType + QueryStringExtractor<Body> + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceSearch<Query, Res>
			{
				let matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				self.0.get(&format!("{}/search", self.1))
					.extend_route_matcher(matcher)
					.with_query_string_extractor::<Query>()
					.to(|state| search_handler::<Handler, Query, Res>(state));
			}
			
			fn create<Handler, Body, Res>(&mut self)
			where
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceCreate<Body, Res>
			{
				let accept_matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Body::supported_types().into();
				self.0.post(&self.1)
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.to(|state| create_handler::<Handler, Body, Res>(state));
			}

			fn update_all<Handler, Body, Res>(&mut self)
			where
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceUpdateAll<Body, Res>
			{
				let accept_matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Body::supported_types().into();
				self.0.put(&self.1)
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.to(|state| update_all_handler::<Handler, Body, Res>(state));
			}

			fn update<Handler, ID, Body, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceUpdate<ID, Body, Res>
			{
				let accept_matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Body::supported_types().into();
				self.0.put(&format!("{}/:id", self.1))
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| update_handler::<Handler, ID, Body, Res>(state));
			}

			fn delete_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceDeleteAll<Res>
			{
				let matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				self.0.delete(&self.1)
					.extend_route_matcher(matcher)
					.to(|state| delete_all_handler::<Handler, Res>(state));
			}

			fn delete<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceDelete<ID, Res>
			{
				let matcher : MaybeMatchAcceptHeader = Res::accepted_types().into();
				self.0.delete(&format!("{}/:id", self.1))
					.extend_route_matcher(matcher)
					.with_path_extractor::<PathExtractor<ID>>()
					.to(|state| delete_handler::<Handler, ID, Res>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
