
#[macro_export]
macro_rules! rest_struct {
	($struct_name:ident { $($field_id:ident : $field_ty:ty),* }) => {
		#[derive(serde::Deserialize, serde::Serialize)]
		struct $struct_name
		{
			$($field_id : $field_ty),*
		}
	}
}

#[macro_export]
macro_rules! rest_resource {
	($res_name:ident) => {
		struct $res_name;

		impl ::gotham_restful::Resource for $res_name
		{
			fn name() -> String
			{
				stringify!($res_name).to_string()
			}

			fn setup<D : ::gotham_restful::DrawResourceRoutes>(mut route : D)
			{
				unimplemented!();
			}
		}
	}
}
