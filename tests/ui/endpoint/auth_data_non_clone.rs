use gotham_restful::*;
use serde::Deserialize;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[derive(Deserialize)]
struct AuthData {
	iat: u64,
	exp: u64
}

#[read_all]
async fn read_all(auth: AuthStatus<AuthData>) -> Result<NoContent, AuthError> {
	auth.ok()?;
	Ok(NoContent::default())
}

fn main() {}
