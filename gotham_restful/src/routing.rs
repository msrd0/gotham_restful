use crate::{
	resource::*,
	result::{ResourceError, ResourceResult, Response},
	RequestBody,
	StatusCode
};
#[cfg(feature = "openapi")]
use crate::OpenapiRouter;

use futures_core::future::Future;
use futures_util::{future, future::FutureExt};
use gotham::{
	handler::{HandlerError, HandlerFuture, IntoHandlerError, IntoHandlerFuture},
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
use gotham::hyper::{
	body::to_bytes,
	header::CONTENT_TYPE,
	Body,
	HeaderMap,
	Method
};
use mime::{Mime, APPLICATION_JSON};
use std::{
	panic::RefUnwindSafe,
	pin::Pin
};

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
	fn read_all<Handler : ResourceReadAll>(&mut self)
	where
		dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send;
	
	fn read<Handler : ResourceRead>(&mut self)
	where
		dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send;
	
	fn search<Handler : ResourceSearch>(&mut self)
	where
		dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send;
	
	fn create<Handler : ResourceCreate>(&mut self)
	where
		Handler::Res : Send + 'static,
		Handler::Body : 'static;
	
	fn update_all<Handler : ResourceUpdateAll>(&mut self)
	where
		Handler::Res : Send + 'static,
		Handler::Body : 'static;
	
	fn update<Handler : ResourceUpdate>(&mut self)
	where
		Handler::Res : Send + 'static,
		Handler::Body : 'static;
	
	fn delete_all<Handler : ResourceDeleteAll>(&mut self)
	where
		dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send;
	
	fn delete<Handler : ResourceDelete>(&mut self)
	where
		dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send;
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

fn to_handler_future<F, R>(mut state : State, get_result : F) -> Pin<Box<HandlerFuture>>
where
	F : FnOnce(&mut State) -> R,
	R : ResourceResult,
	dyn Future<Output = Result<Response, R::Err>> : Send
{
	get_result(&mut state).into_response()
		.then(|res|
			match res {
				Ok(res) => {
					let r = response_from(res, &state);
					(state, r).into_handler_future()
				},
				Err(e) => future::err((state, e.into_handler_error())).boxed()
			}
		).boxed()
}

async fn body_to_res<B, F, R>(state : &mut State, get_result : F) -> Result<gotham::hyper::Response<Body>, HandlerError>
where
	B : RequestBody,
	F : FnOnce(&mut State, B) -> R,
	R : ResourceResult
{
	let body = to_bytes(Body::take_from(&mut state)).await;
	
	let body = match body {
		Ok(body) => body,
		Err(e) => return Err(e.into_handler_error())
	};
	
	let content_type : Mime = match HeaderMap::borrow_from(&state).get(CONTENT_TYPE) {
		Some(content_type) => content_type.to_str().unwrap().parse().unwrap(),
		None => {
			let res = create_empty_response(&state, StatusCode::UNSUPPORTED_MEDIA_TYPE);
			return Ok(res)
		}
	};
	
	let res = {
		let body = match B::from_body(body, content_type) {
			Ok(body) => body,
			Err(e) => return {
				let error : ResourceError = e.into();
				match serde_json::to_string(&error) {
					Ok(json) => {
						let res = create_response(&state, StatusCode::BAD_REQUEST, APPLICATION_JSON, json);
						Ok(res)
					},
					Err(e) => Err(e.into_handler_error())
				}
			}
		};
		get_result(&mut state, body)
	};

	let res = res.into_response().await;
	match res {
		Ok(res) => {
			let r = response_from(res, &state);
			Ok(r)
		},
		Err(e) => Err(e.into_handler_error())
	}
}

fn handle_with_body<B, F, R>(mut state : State, get_result : F) -> Pin<Box<HandlerFuture>>
where
	B : RequestBody + 'static,
	F : FnOnce(&mut State, B) -> R + Send + 'static,
	R : ResourceResult + Send + 'static
{
	body_to_res(&mut state, get_result)
		.then(|res| match res {
			Ok(ok) => future::ok((state, ok)),
			Err(err) => future::err((state, err))
		})
		.boxed()
}

fn read_all_handler<Handler : ResourceReadAll>(state : State) -> Pin<Box<HandlerFuture>>
where
	dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
{
	to_handler_future(state, |state| Handler::read_all(state))
}

fn read_handler<Handler : ResourceRead>(state : State) -> Pin<Box<HandlerFuture>>
where
	dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
{
	let id = {
		let path : &PathExtractor<Handler::ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	to_handler_future(state, |state| Handler::read(state, id))
}

fn search_handler<Handler : ResourceSearch>(mut state : State) -> Pin<Box<HandlerFuture>>
where
	dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
{
	let query = Handler::Query::take_from(&mut state);
	to_handler_future(state, |state| Handler::search(state, query))
}

fn create_handler<Handler : ResourceCreate>(state : State) -> Pin<Box<HandlerFuture>>
where
	Handler::Res : Send + 'static,
	Handler::Body : 'static
{
	handle_with_body::<Handler::Body, _, _>(state, |state, body| Handler::create(state, body))
}

fn update_all_handler<Handler : ResourceUpdateAll>(state : State) -> Pin<Box<HandlerFuture>>
where
	Handler::Res : Send + 'static,
	Handler::Body : 'static
{
	handle_with_body::<Handler::Body, _, _>(state, |state, body| Handler::update_all(state, body))
}

fn update_handler<Handler : ResourceUpdate>(state : State) -> Pin<Box<HandlerFuture>>
where
	Handler::Res : Send + 'static,
	Handler::Body : 'static
{
	let id = {
		let path : &PathExtractor<Handler::ID> = PathExtractor::borrow_from(&state);
		path.id.clone()
	};
	handle_with_body::<Handler::Body, _, _>(state, |state, body| Handler::update(state, id, body))
}

fn delete_all_handler<Handler : ResourceDeleteAll>(state : State) -> Pin<Box<HandlerFuture>>
where
	dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
{
	to_handler_future(state, |state| Handler::delete_all(state))
}

fn delete_handler<Handler : ResourceDelete>(state : State) -> Pin<Box<HandlerFuture>>
where
	dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
{
	let id = {
		let path : &PathExtractor<Handler::ID> = PathExtractor::borrow_from(&state);
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
			fn read_all<Handler : ResourceReadAll>(&mut self)
			where
				dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
			{
				let matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				self.0.get(&self.1)
					.extend_route_matcher(matcher)
					.to(|state| read_all_handler::<Handler>(state));
			}

			fn read<Handler : ResourceRead>(&mut self)
			where
				dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
			{
				let matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				self.0.get(&format!("{}/:id", self.1))
					.extend_route_matcher(matcher)
					.with_path_extractor::<PathExtractor<Handler::ID>>()
					.to(|state| read_handler::<Handler>(state));
			}
			
			fn search<Handler : ResourceSearch>(&mut self)
			where
				dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
			{
				let matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				self.0.get(&format!("{}/search", self.1))
					.extend_route_matcher(matcher)
					.with_query_string_extractor::<Handler::Query>()
					.to(|state| search_handler::<Handler>(state));
			}
			
			fn create<Handler : ResourceCreate>(&mut self)
			where
				Handler::Res : Send + 'static,
				Handler::Body : 'static
			{
				let accept_matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Handler::Body::supported_types().into();
				self.0.post(&self.1)
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.to(|state| create_handler::<Handler>(state));
			}

			fn update_all<Handler : ResourceUpdateAll>(&mut self)
			where
				Handler::Res : Send + 'static,
				Handler::Body : 'static
			{
				let accept_matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Handler::Body::supported_types().into();
				self.0.put(&self.1)
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.to(|state| update_all_handler::<Handler>(state));
			}

			fn update<Handler : ResourceUpdate>(&mut self)
			where
				Handler::Res : Send + 'static,
				Handler::Body : 'static
			{
				let accept_matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				let content_matcher : MaybeMatchContentTypeHeader = Handler::Body::supported_types().into();
				self.0.put(&format!("{}/:id", self.1))
					.extend_route_matcher(accept_matcher)
					.extend_route_matcher(content_matcher)
					.with_path_extractor::<PathExtractor<Handler::ID>>()
					.to(|state| update_handler::<Handler>(state));
			}

			fn delete_all<Handler : ResourceDeleteAll>(&mut self)
			where
				dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
			{
				let matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				self.0.delete(&self.1)
					.extend_route_matcher(matcher)
					.to(|state| delete_all_handler::<Handler>(state));
			}

			fn delete<Handler : ResourceDelete>(&mut self)
			where
				dyn Future<Output = Result<Response, <Handler::Res as ResourceResult>::Err>> : Send
			{
				let matcher : MaybeMatchAcceptHeader = Handler::Res::accepted_types().into();
				self.0.delete(&format!("{}/:id", self.1))
					.extend_route_matcher(matcher)
					.with_path_extractor::<PathExtractor<Handler::ID>>()
					.to(|state| delete_handler::<Handler>(state));
			}
		}
	}
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
