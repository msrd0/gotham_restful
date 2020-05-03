#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate log;

use fake::{faker::internet::en::Username, Fake};
use gotham::{
	middleware::logger::RequestLogger,
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	state::State
};
use gotham_restful::*;
use log::LevelFilter;
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Config, Root},
	encode::pattern::PatternEncoder
};
use serde::{Deserialize, Serialize};

#[derive(Resource)]
#[rest_resource(ReadAll, Read, Search, Create, DeleteAll, Delete, Update, UpdateAll)]
struct Users
{
}

#[derive(Resource)]
#[rest_resource(ReadAll)]
struct Auth
{
}

#[derive(Deserialize, OpenapiType, Serialize, StateData, StaticResponseExtender)]
struct User
{
	username : String
}

#[rest_read_all(Users)]
fn read_all() -> Success<Vec<Option<User>>>
{
	vec![Username().fake(), Username().fake()]
		.into_iter()
		.map(|username| Some(User { username }))
		.collect::<Vec<Option<User>>>()
		.into()
}

#[rest_read(Users)]
fn read(id : u64) -> Success<User>
{
	let username : String = Username().fake();
	User { username: format!("{}{}", username, id) }.into()
}

#[rest_search(Users)]
fn search(query : User) -> Success<User>
{
	query.into()
}

#[rest_create(Users)]
fn create(body : User)
{
	info!("Created User: {}", body.username);
}

#[rest_update_all(Users)]
fn update_all(body : Vec<User>)
{
	info!("Changing all Users to {:?}", body.into_iter().map(|u| u.username).collect::<Vec<String>>());
}

#[rest_update(Users)]
fn update(id : u64, body : User)
{
	info!("Change User {} to {}", id, body.username);
}

#[rest_delete_all(Users)]
fn delete_all()
{
	info!("Delete all Users");
}

#[rest_delete(Users)]
fn delete(id : u64)
{
	info!("Delete User {}", id);
}

#[rest_read_all(Auth)]
fn auth_read_all(auth : AuthStatus<()>) -> AuthSuccess<String>
{
	match auth {
		AuthStatus::Authenticated(data) => Ok(format!("{:?}", data)),
		_ => Err(Forbidden)
	}
}

const ADDR : &str = "127.0.0.1:18080";

#[derive(Clone, Default)]
struct Handler;
impl<T> AuthHandler<T> for Handler
{
	fn jwt_secret<F : FnOnce() -> Option<T>>(&self, _state : &mut State, _decode_data : F) -> Option<Vec<u8>>
	{
		None
	}
}

fn main()
{
	let encoder = PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f %Z)} [{l}] {M} - {m}\n");
	let config = Config::builder()
		.appender(
			Appender::builder()
				.build("stdout", Box::new(
					ConsoleAppender::builder()
						.encoder(Box::new(encoder))
						.build()
				)))
		.build(Root::builder().appender("stdout").build(LevelFilter::Info))
		.unwrap();
	log4rs::init_config(config).unwrap();
	
	let auth = <AuthMiddleware<(), Handler>>::from_source(AuthSource::AuthorizationHeader);
	let logging = RequestLogger::new(log::Level::Info);
	let (chain, pipelines) = single_pipeline(
		new_pipeline()
			.add(auth)
			.add(logging)
			.build()
	);

	gotham::start(ADDR, build_router(chain, pipelines, |route| {
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
	}));
	println!("Gotham started on {} for testing", ADDR);
}

