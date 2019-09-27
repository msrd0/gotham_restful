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

impl IndexResource<Success<Vec<User>>> for Users
{
	fn index(_state : &mut State) -> Success<Vec<User>>
	{
		vec![Username().fake(), Username().fake()]
			.into_iter()
			.map(|username| User { username })
			.collect::<Vec<User>>()
			.into()
	}
}

impl GetResource<u64, Success<User>> for Users
{
	fn get(_state : &mut State, id : u64) -> Success<User>
	{
		let username : String = Username().fake();
		User { username: format!("{}{}", username, id) }.into()
	}
}

impl CreateResource<User, Success<()>> for Users
{
	fn create(_state : &mut State, body : User) -> Success<()>
	{
		info!("Created User: {}", body.username);
		().into()
	}
}

impl ChangeAllResource<Vec<User>, Success<()>> for Users
{
	fn change_all(_state : &mut State, body : Vec<User>) -> Success<()>
	{
		info!("Changing all Users to {:?}", body.into_iter().map(|u| u.username).collect::<Vec<String>>());
		().into()
	}
}

impl ChangeResource<u64, User, Success<()>> for Users
{
	fn change(_state : &mut State, id : u64, body : User) -> Success<()>
	{
		info!("Change User {} to {}", id, body.username);
		().into()
	}
}

impl Resource for Users
{
	fn setup<D : DrawResourceRoutes>(mut route : D)
	{
		route.index::<_, Self>();
		route.get::<_, _, Self>();
		route.create::<_, _, Self>();
		route.change_all::<_, _, Self>();
		route.change::<_, _, _, Self>();
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

