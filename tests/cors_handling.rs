#![cfg(feature = "cors")]
use gotham::{
	hyper::{body::Body, client::connect::Connect, header::*, StatusCode},
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	test::{Server, TestRequest, TestServer}
};
use gotham_restful::{
	change_all,
	cors::{Headers, Origin},
	read_all, CorsConfig, DrawResources, Raw, Resource
};
use mime::TEXT_PLAIN;

#[derive(Resource)]
#[resource(read_all, change_all)]
struct FooResource;

#[read_all]
fn read_all() {}

#[change_all]
fn change_all(_body: Raw<Vec<u8>>) {}

fn test_server(cfg: CorsConfig) -> TestServer {
	let (chain, pipeline) = single_pipeline(new_pipeline().add(cfg).build());
	TestServer::new(build_router(chain, pipeline, |router| router.resource::<FooResource>("/foo"))).unwrap()
}

fn test_response<TS, C>(req: TestRequest<TS, C>, origin: Option<&str>, vary: Option<&str>, credentials: bool)
where
	TS: Server + 'static,
	C: Connect + Clone + Send + Sync + 'static
{
	let res = req
		.with_header(ORIGIN, "http://example.org".parse().unwrap())
		.perform()
		.unwrap();
	assert_eq!(res.status(), StatusCode::NO_CONTENT);
	let headers = res.headers();
	println!("{}", headers.keys().map(|name| name.as_str()).collect::<Vec<_>>().join(","));
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_ALLOW_ORIGIN)
			.and_then(|value| value.to_str().ok())
			.as_deref(),
		origin
	);
	assert_eq!(headers.get(VARY).and_then(|value| value.to_str().ok()).as_deref(), vary);
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
			.and_then(|value| value.to_str().ok())
			.map(|value| value == "true")
			.unwrap_or(false),
		credentials
	);
	assert!(headers.get(ACCESS_CONTROL_MAX_AGE).is_none());
}

fn test_preflight(server: &TestServer, method: &str, origin: Option<&str>, vary: &str, credentials: bool, max_age: u64) {
	let res = server
		.client()
		.options("http://example.org/foo")
		.with_header(ACCESS_CONTROL_REQUEST_METHOD, method.parse().unwrap())
		.with_header(ORIGIN, "http://example.org".parse().unwrap())
		.perform()
		.unwrap();
	assert_eq!(res.status(), StatusCode::NO_CONTENT);
	let headers = res.headers();
	println!("{}", headers.keys().map(|name| name.as_str()).collect::<Vec<_>>().join(","));
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_ALLOW_METHODS)
			.and_then(|value| value.to_str().ok())
			.as_deref(),
		Some(method)
	);
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_ALLOW_ORIGIN)
			.and_then(|value| value.to_str().ok())
			.as_deref(),
		origin
	);
	assert_eq!(headers.get(VARY).and_then(|value| value.to_str().ok()).as_deref(), Some(vary));
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
			.and_then(|value| value.to_str().ok())
			.map(|value| value == "true")
			.unwrap_or(false),
		credentials
	);
	assert_eq!(
		headers
			.get(ACCESS_CONTROL_MAX_AGE)
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.parse().ok()),
		Some(max_age)
	);
}

fn test_preflight_headers(
	server: &TestServer,
	method: &str,
	request_headers: Option<&str>,
	allowed_headers: Option<&str>,
	vary: &str
) {
	let client = server.client();
	let mut res = client
		.options("http://example.org/foo")
		.with_header(ACCESS_CONTROL_REQUEST_METHOD, method.parse().unwrap())
		.with_header(ORIGIN, "http://example.org".parse().unwrap());
	if let Some(hdr) = request_headers {
		res = res.with_header(ACCESS_CONTROL_REQUEST_HEADERS, hdr.parse().unwrap());
	}
	let res = res.perform().unwrap();
	assert_eq!(res.status(), StatusCode::NO_CONTENT);
	let headers = res.headers();
	println!("{}", headers.keys().map(|name| name.as_str()).collect::<Vec<_>>().join(","));
	if let Some(hdr) = allowed_headers {
		assert_eq!(
			headers
				.get(ACCESS_CONTROL_ALLOW_HEADERS)
				.and_then(|value| value.to_str().ok())
				.as_deref(),
			Some(hdr)
		)
	} else {
		assert!(!headers.contains_key(ACCESS_CONTROL_ALLOW_HEADERS));
	}
	assert_eq!(headers.get(VARY).and_then(|value| value.to_str().ok()).as_deref(), Some(vary));
}

#[test]
fn cors_origin_none() {
	let cfg = Default::default();
	let server = test_server(cfg);

	test_preflight(&server, "PUT", None, "access-control-request-method", false, 0);

	test_response(server.client().get("http://example.org/foo"), None, None, false);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		None,
		None,
		false
	);
}

#[test]
fn cors_origin_star() {
	let cfg = CorsConfig {
		origin: Origin::Star,
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight(&server, "PUT", Some("*"), "access-control-request-method", false, 0);

	test_response(server.client().get("http://example.org/foo"), Some("*"), None, false);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		Some("*"),
		None,
		false
	);
}

#[test]
fn cors_origin_single() {
	let cfg = CorsConfig {
		origin: Origin::Single("https://foo.com".to_owned()),
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight(
		&server,
		"PUT",
		Some("https://foo.com"),
		"access-control-request-method",
		false,
		0
	);

	test_response(
		server.client().get("http://example.org/foo"),
		Some("https://foo.com"),
		None,
		false
	);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		Some("https://foo.com"),
		None,
		false
	);
}

#[test]
fn cors_origin_copy() {
	let cfg = CorsConfig {
		origin: Origin::Copy,
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight(
		&server,
		"PUT",
		Some("http://example.org"),
		"access-control-request-method,origin",
		false,
		0
	);

	test_response(
		server.client().get("http://example.org/foo"),
		Some("http://example.org"),
		Some("origin"),
		false
	);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		Some("http://example.org"),
		Some("origin"),
		false
	);
}

#[test]
fn cors_headers_none() {
	let cfg = Default::default();
	let server = test_server(cfg);

	test_preflight_headers(&server, "PUT", None, None, "access-control-request-method");
	test_preflight_headers(&server, "PUT", Some("Content-Type"), None, "access-control-request-method");
}

#[test]
fn cors_headers_list() {
	let cfg = CorsConfig {
		headers: Headers::List(vec![CONTENT_TYPE]),
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight_headers(&server, "PUT", None, Some("content-type"), "access-control-request-method");
	test_preflight_headers(
		&server,
		"PUT",
		Some("content-type"),
		Some("content-type"),
		"access-control-request-method"
	);
}

#[test]
fn cors_headers_copy() {
	let cfg = CorsConfig {
		headers: Headers::Copy,
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight_headers(
		&server,
		"PUT",
		None,
		None,
		"access-control-request-method,access-control-request-headers"
	);
	test_preflight_headers(
		&server,
		"PUT",
		Some("content-type"),
		Some("content-type"),
		"access-control-request-method,access-control-request-headers"
	);
}

#[test]
fn cors_credentials() {
	let cfg = CorsConfig {
		origin: Origin::None,
		credentials: true,
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight(&server, "PUT", None, "access-control-request-method", true, 0);

	test_response(server.client().get("http://example.org/foo"), None, None, true);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		None,
		None,
		true
	);
}

#[test]
fn cors_max_age() {
	let cfg = CorsConfig {
		origin: Origin::None,
		max_age: 31536000,
		..Default::default()
	};
	let server = test_server(cfg);

	test_preflight(&server, "PUT", None, "access-control-request-method", false, 31536000);

	test_response(server.client().get("http://example.org/foo"), None, None, false);
	test_response(
		server.client().put("http://example.org/foo", Body::empty(), TEXT_PLAIN),
		None,
		None,
		false
	);
}
