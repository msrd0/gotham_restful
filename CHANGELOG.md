# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org).

## [Unreleased]

## [0.4.5] - 2021-07-26
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.55](https://github.com/Redocly/redoc/releases/tag/v2.0.0-rc.55)

## [0.4.4] - 2021-06-25
### Changed
 - The `IntoResponseError` trait is now implemented for all errors that can be converted to `anyhow::Error`, not just `std::error::Error`

## [0.4.3] - 2021-06-12
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.54](https://github.com/Redocly/redoc/releases/tag/v2.0.0-rc.54),
   which, among other changes, reduces the file size

### Changed
 - Use the `lazy-regex` crate for all regular expressions

## [0.4.2] - 2021-05-24
### Changed
 - Improved error message when `T` does not implement `Clone` in `AuthStatus<T>` (`auth` feature only)
 - Set viewport in ReDoc HTML so that it can be viewed on mobile devices (`openapi` feature only)

## [0.4.1] - 2021-05-20
### Changed
 - The readme is now extracted using [cargo-doc2readme] to support doc links

### Removed
 - `dbg!` call inside the auth checking code

## [0.4.0] - 2021-05-19
### Changed
 - Rename endpoint macro `change` to `update` and `remove` to `delete`
 - All endpoints with a known operation verb (like `read` or `update`) now have an auto-generated operation id (`openapi` feature only)
 - Endpoint macros now place the rustdoc into the operation's description (`openapi` feature only)
 - Update `openapi_type` crate to 0.2
 - Replace swagger-ui with redoc and a semi-dark theme (`openapi` feature only)
 - Rename `get_openapi` to `openapi_spec` and `swagger_ui` to `openapi_doc` (that now serves redoc)

## [0.3.0] - 2021-03-21
### Changed
 - The `OpenapiType` trait and derive macro have been "outsourced" into the `openapi_type` crate
 - Updated `gotham` to version 0.6

## Previous releases
Previous releases were hosted on GitLab. Please see https://gitlab.com/msrd0/gotham-restful/-/blob/master/CHANGELOG.md

 [cargo-doc2readme]: https://github.com/msrd0/cargo-doc2readme
