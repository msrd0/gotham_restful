#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate log;

use fake::{faker::internet::en::Username, Fake};
use gotham::{
	hyper::header::CONTENT_TYPE,
	middleware::logger::RequestLogger,
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	state::State
};
use gotham_restful::{cors::*, *};
use serde::{Deserialize, Serialize};

#[derive(Resource)]
#[resource(read_all, read, search, create, update_all, update, remove, remove_all)]
struct Users {}

#[derive(Resource)]
#[resource(auth_read_all)]
struct Auth {}

#[derive(Deserialize, OpenapiType, Serialize, StateData, StaticResponseExtender)]
struct User {
	username: String
}

#[read_all]
fn read_all() -> Success<Vec<Option<User>>> {
	vec![Username().fake(), Username().fake()]
		.into_iter()
		.map(|username| Some(User { username }))
		.collect::<Vec<Option<User>>>()
		.into()
}

#[read]
fn read(id: u64) -> Success<User> {
	let username: String = Username().fake();
	User {
		username: format!("{}{}", username, id)
	}
	.into()
}

#[search]
fn search(query: User) -> Success<User> {
	query.into()
}

#[create]
fn create(body: User) {
	info!("Created User: {}", body.username);
}

#[change_all]
fn update_all(body: Vec<User>) {
	info!(
		"Changing all Users to {:?}",
		body.into_iter().map(|u| u.username).collect::<Vec<String>>()
	);
}

#[change]
fn update(id: u64, body: User) {
	info!("Change User {} to {}", id, body.username);
}

#[remove_all]
fn remove_all() {
	info!("Delete all Users");
}

#[remove]
fn remove(id: u64) {
	info!("Delete User {}", id);
}

#[read_all]
fn auth_read_all(auth: AuthStatus<()>) -> AuthSuccess<String> {
	match auth {
		AuthStatus::Authenticated(data) => Ok(format!("{:?}", data)),
		_ => Err(Forbidden)
	}
}

const ADDR: &str = "127.0.0.1:18080";

#[derive(Clone, Default)]
struct Handler;
impl<T> AuthHandler<T> for Handler {
	fn jwt_secret<F: FnOnce() -> Option<T>>(&self, _state: &mut State, _decode_data: F) -> Option<Vec<u8>> {
		None
	}
}

fn main() {
	pretty_env_logger::init_timed();

	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: Headers::List(vec![CONTENT_TYPE]),
		credentials: true,
		..Default::default()
	};

	let auth = <AuthMiddleware<(), Handler>>::from_source(AuthSource::AuthorizationHeader);
	let logging = RequestLogger::new(log::Level::Info);
	let (chain, pipelines) = single_pipeline(new_pipeline().add(auth).add(logging).add(cors).build());

	gotham::start(
		ADDR,
		build_router(chain, pipelines, |route| {
			let info = OpenapiInfo {
				title: "Users Example".to_owned(),
				version: "0.0.1".to_owned(),
				urls: vec![format!("http://{}", ADDR)]
			};
			route.with_openapi(info, |mut route| {
				route.resource::<Users>("users");
				route.resource::<Auth>("auth");
				route.get_openapi("openapi");
			});
		})
	);
	println!("Gotham started on {} for testing", ADDR);
}
