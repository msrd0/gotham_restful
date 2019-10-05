#[macro_use] extern crate log;
#[macro_use] extern crate gotham_restful_derive;

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

rest_resource!{Users, route => {
	route.read_all::<Self, _>();
	route.read::<Self, _, _>();
	route.create::<Self, _, _>();
	route.update_all::<Self, _, _>();
	route.update::<Self, _, _, _>();
}}

#[derive(Deserialize, OpenapiType, Serialize)]
struct User
{
	username : String
}

#[rest_read_all(Users)]
fn read_all(_state : &mut State) -> Success<Vec<Option<User>>>
{
	vec![Username().fake(), Username().fake()]
		.into_iter()
		.map(|username| Some(User { username }))
		.collect::<Vec<Option<User>>>()
		.into()
}

#[rest_read(Users)]
fn read(_state : &mut State, id : u64) -> Success<User>
{
	let username : String = Username().fake();
	User { username: format!("{}{}", username, id) }.into()
}

#[rest_create(Users)]
fn create(_state : &mut State, body : User) -> NoContent
{
	info!("Created User: {}", body.username);
	().into()
}

#[rest_update_all(Users)]
fn update_all(_state : &mut State, body : Vec<User>) -> NoContent
{
	info!("Changing all Users to {:?}", body.into_iter().map(|u| u.username).collect::<Vec<String>>());
	().into()
}

#[rest_update(Users)]
fn update(_state : &mut State, id : u64, body : User) -> NoContent
{
	info!("Change User {} to {}", id, body.username);
	().into()
}

#[rest_delete_all(Users)]
fn delete_all(_state : &mut State) -> NoContent
{
	info!("Delete all Users");
	().into()
}

#[rest_delete(Users)]
fn delete(_state : &mut State, id : u64) -> NoContent
{
	info!("Delete User {}", id);
	().into()
}

const ADDR : &str = "127.0.0.1:18080";

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
	
	let logging = RequestLogger::new(log::Level::Info);
	let (chain, pipelines) = single_pipeline(
		new_pipeline()
			.add(logging)
			.build()
	);

	gotham::start(ADDR, build_router(chain, pipelines, |route| {
		route.with_openapi("Users Example", "0.0.1", format!("http://{}", ADDR), |mut route| {
			route.resource::<Users, _>("users");
			route.get_openapi("openapi");
		});
	}));
	println!("Gotham started on {} for testing", ADDR);
}

