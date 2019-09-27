extern crate log;
#[macro_use]
extern crate serde_derive;

use fake::{faker::internet::en::Username, Fake};
use gotham::{
	middleware::logger::RequestLogger,
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	state::State
};
use gotham_restful::{DrawResources, DrawResourceRoutes, IndexResource, Resource, Success};

struct Users;

#[derive(Serialize)]
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

impl Resource for Users
{
	fn setup<D : DrawResourceRoutes>(mut route : D)
	{
		route.index::<_, Self>();
	}
}

const ADDR : &str = "127.0.0.1:18080";

fn main()
{
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

