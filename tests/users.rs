#[macro_use]
extern crate serde_derive;

use fake::{faker::internet::en::Username, Fake};
use gotham::{
	router::builder::build_simple_router,
	state::State
};
use gotham_restful::{DrawResources, DrawResourceRoutes, IndexResource, Resource, ResourceResult, Success};
use reqwest;
use std::{
	thread,
	time::Duration
};

struct Users;

#[derive(Serialize)]
struct User
{
	username : String
}

impl IndexResource<Success<Vec<User>>> for Users
{
	fn index(state : &mut State) -> Success<Vec<User>>
	{
		panic!("Index handler called");
		vec![Username().fake(), Username().fake()]
			.into_iter()
			.map(|username| User { username })
			.collect::<Vec<User>>()
			.into()
	}
}

impl Resource for Users
{
	fn setup<D : DrawResourceRoutes>(route : D)
	{
	}
}

const ADDR : &str = "127.0.0.1:18080";

pub fn setup()
{
	thread::spawn(|| {
		gotham::start(ADDR, build_simple_router(|route| {
			route.resource::<Users, _>("users");
		}));
		panic!("Gotham started on {} for testing", ADDR);
	});
	thread::sleep(Duration::from_millis(1000));
}

#[test]
fn user_index()
{
	setup();
	let answer = reqwest::get(&format!("http://{}/users", ADDR))
		.expect("Unable to execute GET request")
		.text()
		.expect("Unable to get body of GET request");
	panic!("answer: {}", answer);
}
