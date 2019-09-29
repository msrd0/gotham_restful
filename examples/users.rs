#[macro_use] extern crate log;
#[macro_use] extern crate serde;

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

struct Users;

#[derive(Deserialize, Serialize)]
struct User
{
	username : String
}

impl ResourceReadAll<Success<Vec<User>>> for Users
{
	fn read_all(_state : &mut State) -> Success<Vec<User>>
	{
		vec![Username().fake(), Username().fake()]
			.into_iter()
			.map(|username| User { username })
			.collect::<Vec<User>>()
			.into()
	}
}

impl ResourceRead<u64, Success<User>> for Users
{
	fn read(_state : &mut State, id : u64) -> Success<User>
	{
		let username : String = Username().fake();
		User { username: format!("{}{}", username, id) }.into()
	}
}

impl ResourceCreate<User, Success<()>> for Users
{
	fn create(_state : &mut State, body : User) -> Success<()>
	{
		info!("Created User: {}", body.username);
		().into()
	}
}

impl ResourceUpdateAll<Vec<User>, Success<()>> for Users
{
	fn update_all(_state : &mut State, body : Vec<User>) -> Success<()>
	{
		info!("Changing all Users to {:?}", body.into_iter().map(|u| u.username).collect::<Vec<String>>());
		().into()
	}
}

impl ResourceUpdate<u64, User, Success<()>> for Users
{
	fn update(_state : &mut State, id : u64, body : User) -> Success<()>
	{
		info!("Change User {} to {}", id, body.username);
		().into()
	}
}

impl ResourceDeleteAll<Success<()>> for Users
{
	fn delete_all(_state : &mut State) -> Success<()>
	{
		info!("Delete all Users");
		().into()
	}
}

impl ResourceDelete<u64, Success<()>> for Users
{
	fn delete(_state : &mut State, id : u64) -> Success<()>
	{
		info!("Delete User {}", id);
		().into()
	}
}

impl Resource for Users
{
	fn setup<D : DrawResourceRoutes>(mut route : D)
	{
		route.read_all::<Self, _>();
		route.read::<Self, _, _>();
		route.create::<Self, _, _>();
		route.update_all::<Self, _, _>();
		route.update::<Self, _, _, _>();
	}
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
		route.resource::<Users, _>("users");
	}));
	println!("Gotham started on {} for testing", ADDR);
}

