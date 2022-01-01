#[cfg(feature = "openapi")]
use crate::openapi::{
	builder::{OpenapiBuilder, OpenapiInfo},
	router::OpenapiRouter
};
use crate::{response::ResourceError, Endpoint, FromBody, IntoResponse, Resource, Response};
#[cfg(feature = "cors")]
use gotham::router::route::matcher::AccessControlRequestMethodMatcher;
use gotham::{
	handler::HandlerError,
	helpers::http::response::{create_empty_response, create_response},
	hyper::{body::to_bytes, header::CONTENT_TYPE, Body, HeaderMap, Method, StatusCode},
	mime::{Mime, APPLICATION_JSON},
	pipeline::PipelineHandleChain,
	prelude::*,
	router::{
		builder::{RouterBuilder, ScopeBuilder},
		route::matcher::{AcceptHeaderRouteMatcher, ContentTypeHeaderRouteMatcher, RouteMatcher},
		RouteNonMatch
	},
	state::{FromState, State}
};
#[cfg(feature = "openapi")]
use openapi_type::OpenapiType;
use std::{any::TypeId, panic::RefUnwindSafe};

/// Allow us to extract an id from a path.
#[derive(Clone, Copy, Debug, Deserialize, StateData, StaticResponseExtender)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
pub struct PathExtractor<ID: RefUnwindSafe + Send + 'static> {
	pub id: ID
}

/// This trait adds the `with_openapi` method to gotham's routing. It turns the default
/// router into one that will only allow RESTful resources, but record them and generate
/// an OpenAPI specification on request.
#[cfg(feature = "openapi")]
pub trait WithOpenapi<D> {
	fn with_openapi<F>(&mut self, info: OpenapiInfo, block: F)
	where
		F: FnOnce(OpenapiRouter<'_, D>);
}

/// This trait adds the `resource` method to gotham's routing. It allows you to register
/// any RESTful [Resource] with a path.
#[_private_openapi_trait(DrawResourcesWithSchema)]
pub trait DrawResources {
	#[openapi_bound("R: crate::ResourceWithSchema")]
	#[non_openapi_bound("R: crate::Resource")]
	fn resource<R>(&mut self, path: &str);
}

/// This trait allows to draw routes within an resource. Use this only inside the
/// [Resource::setup] method.
#[_private_openapi_trait(DrawResourceRoutesWithSchema)]
pub trait DrawResourceRoutes {
	#[openapi_bound("E: crate::EndpointWithSchema")]
	#[non_openapi_bound("E: crate::Endpoint")]
	fn endpoint<E: 'static>(&mut self);
}

fn response_from(res: Response, state: &State) -> gotham::hyper::Response<Body> {
	let mut r = create_empty_response(state, res.status);
	let headers = r.headers_mut();
	if let Some(mime) = res.mime {
		headers.insert(CONTENT_TYPE, mime.as_ref().parse().unwrap());
	}
	let mut last_name = None;
	for (name, value) in res.headers {
		if name.is_some() {
			last_name = name;
		}
		// this unwrap is safe: the first item will always be Some
		let name = last_name.clone().unwrap();
		headers.insert(name, value);
	}

	let method = Method::borrow_from(state);
	if method != Method::HEAD {
		*r.body_mut() = res.body;
	}

	#[cfg(feature = "cors")]
	crate::cors::handle_cors(state, &mut r);

	r
}

async fn endpoint_handler<E: Endpoint>(state: &mut State) -> Result<gotham::hyper::Response<Body>, HandlerError>
where
	E: Endpoint,
	<E::Output as IntoResponse>::Err: Into<HandlerError>
{
	trace!("entering endpoint_handler");
	let placeholders = E::Placeholders::take_from(state);
	// workaround for E::Placeholders and E::Param being the same type
	// when fixed remove `Clone` requirement on endpoint
	if TypeId::of::<E::Placeholders>() == TypeId::of::<E::Params>() {
		state.put(placeholders.clone());
	}
	let params = E::Params::take_from(state);

	let body = match E::needs_body() {
		true => {
			let body = to_bytes(Body::take_from(state)).await?;

			let content_type: Mime = match HeaderMap::borrow_from(state).get(CONTENT_TYPE) {
				Some(content_type) => content_type.to_str().unwrap().parse().unwrap(),
				None => {
					debug!("Missing Content-Type: Returning 415 Response");
					let res = create_empty_response(state, StatusCode::UNSUPPORTED_MEDIA_TYPE);
					return Ok(res);
				}
			};

			match E::Body::from_body(body, content_type) {
				Ok(body) => Some(body),
				Err(e) => {
					debug!("Invalid Body: Returning 400 Response");
					let error: ResourceError = e.into();
					let json = serde_json::to_string(&error)?;
					let res = create_response(state, StatusCode::BAD_REQUEST, APPLICATION_JSON, json);
					return Ok(res);
				}
			}
		},
		false => None
	};

	let out = E::handle(state, placeholders, params, body).await;
	let res = out.into_response().await.map_err(Into::into)?;
	debug!("Returning response {:?}", res);
	Ok(response_from(res, state))
}

#[derive(Clone)]
struct MaybeMatchAcceptHeader {
	matcher: Option<AcceptHeaderRouteMatcher>
}

impl RouteMatcher for MaybeMatchAcceptHeader {
	fn is_match(&self, state: &State) -> Result<(), RouteNonMatch> {
		match &self.matcher {
			Some(matcher) => matcher.is_match(state),
			None => Ok(())
		}
	}
}

impl MaybeMatchAcceptHeader {
	fn new(types: Option<Vec<Mime>>) -> Self {
		let types = match types {
			Some(types) if types.is_empty() => None,
			types => types
		};
		Self {
			matcher: types.map(AcceptHeaderRouteMatcher::new)
		}
	}
}

impl From<Option<Vec<Mime>>> for MaybeMatchAcceptHeader {
	fn from(types: Option<Vec<Mime>>) -> Self {
		Self::new(types)
	}
}

#[derive(Clone)]
struct MaybeMatchContentTypeHeader {
	matcher: Option<ContentTypeHeaderRouteMatcher>
}

impl RouteMatcher for MaybeMatchContentTypeHeader {
	fn is_match(&self, state: &State) -> Result<(), RouteNonMatch> {
		match &self.matcher {
			Some(matcher) => matcher.is_match(state),
			None => Ok(())
		}
	}
}

impl MaybeMatchContentTypeHeader {
	fn new(types: Option<Vec<Mime>>) -> Self {
		Self {
			matcher: types.map(|types| ContentTypeHeaderRouteMatcher::new(types).allow_no_type())
		}
	}
}

impl From<Option<Vec<Mime>>> for MaybeMatchContentTypeHeader {
	fn from(types: Option<Vec<Mime>>) -> Self {
		Self::new(types)
	}
}

macro_rules! implDrawResourceRoutes {
	($implType:ident) => {
		#[cfg(feature = "openapi")]
		impl<'a, C, P> WithOpenapi<Self> for $implType<'a, C, P>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn with_openapi<F>(&mut self, info: OpenapiInfo, block: F)
			where
				F: FnOnce(OpenapiRouter<'_, $implType<'a, C, P>>)
			{
				let router = OpenapiRouter {
					router: self,
					scope: None,
					openapi_builder: &mut OpenapiBuilder::new(info)
				};
				block(router);
			}
		}

		impl<'a, C, P> DrawResources for $implType<'a, C, P>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R: Resource>(&mut self, mut path: &str) {
				if path.starts_with('/') {
					path = &path[1..];
				}
				R::setup((self, path));
			}
		}

		impl<'a, C, P> DrawResourceRoutes for (&mut $implType<'a, C, P>, &str)
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn endpoint<E: Endpoint + 'static>(&mut self) {
				let uri = format!("{}/{}", self.1, E::uri());
				debug!("Registering endpoint for {}", uri);
				self.0.associate(&uri, |assoc| {
					assoc
						.request(vec![E::http_method()])
						.add_route_matcher(MaybeMatchAcceptHeader::new(E::Output::accepted_types()))
						.with_path_extractor::<E::Placeholders>()
						.with_query_string_extractor::<E::Params>()
						.to_async_borrowing(endpoint_handler::<E>);

					#[cfg(feature = "cors")]
					if E::http_method() != Method::GET {
						assoc
							.options()
							.add_route_matcher(AccessControlRequestMethodMatcher::new(E::http_method()))
							.to(crate::cors::cors_preflight_handler);
					}
				});
			}
		}
	};
}

implDrawResourceRoutes!(RouterBuilder);
implDrawResourceRoutes!(ScopeBuilder);
