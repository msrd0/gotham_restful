use trybuild::TestCases;

#[test]
fn ui()
{
	let t = TestCases::new();
	
	// always enabled
	t.compile_fail("tests/ui/from_body_enum.rs");
}