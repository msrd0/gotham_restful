use trybuild::TestCases;

#[test]
#[ignore]
fn trybuild_ui() {
	let t = TestCases::new();

	// always enabled
	t.compile_fail("tests/ui/from_body_enum.rs");
	t.compile_fail("tests/ui/method_async_state.rs");
	t.compile_fail("tests/ui/method_for_unknown_resource.rs");
	t.compile_fail("tests/ui/method_no_resource.rs");
	t.compile_fail("tests/ui/method_self.rs");
	t.compile_fail("tests/ui/method_too_few_args.rs");
	t.compile_fail("tests/ui/method_too_many_args.rs");
	t.compile_fail("tests/ui/method_unsafe.rs");
	t.compile_fail("tests/ui/resource_unknown_method.rs");

	// require the openapi feature
	if cfg!(feature = "openapi") {
		t.compile_fail("tests/ui/openapi_type_enum_with_fields.rs");
		t.compile_fail("tests/ui/openapi_type_nullable_non_bool.rs");
		t.compile_fail("tests/ui/openapi_type_rename_non_string.rs");
		t.compile_fail("tests/ui/openapi_type_tuple_struct.rs");
		t.compile_fail("tests/ui/openapi_type_union.rs");
		t.compile_fail("tests/ui/openapi_type_unknown_key.rs");
	}
}
