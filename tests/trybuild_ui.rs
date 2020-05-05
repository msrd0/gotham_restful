use trybuild::TestCases;

#[test]
#[ignore]
fn trybuild_ui()
{
	let t = TestCases::new();
	
	// always enabled
	t.compile_fail("tests/ui/from_body_enum.rs");
}
