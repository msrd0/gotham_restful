# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org).

## Unreleased

## [0.9.0] - 2024-05-14
### Changed
 - The OpenAPI operation id will now be generated from the function name instead of the
   operation verb

### Updated
 - `openapi_type` has been updated to 0.5
 - `jsonwebtoken` has been updated to 9.3

## [0.8.5] - 2024-04-26
### Updated
 - `base64` has been updated to 0.22

## [0.8.4] - 2023-09-13
### Updated
 - Some dependency versions were increased to ensure compatibility with `-Z minimal-versions`

## [0.8.3] - 2023-03-20
### Updated
 - `syn` has been updated to 2.0
 - The requirement on `quote` has been relaxed after `diesel` got updated.

## [0.8.2] - 2023-03-13
### Updated
 - Updated `base64` to 0.21

### Fixed
 - `quote` decided to integrate breaking changes into a patch release, which breaks compiling `diesel`. We now require a non-broken version of quote.

## [0.8.1] - 2023-01-01
### Updated
 - Fixed proc macro UI that regressed between Rust 1.64 and 1.66: The compiler produced unhelpful and misleading suggestions

## [0.8.0] - 2022-11-07
### Updated
 - Updated `diesel` to 2.0

## [0.7.8] - 2022-09-26
### Changed
 - Fixed a compile error when using the `openapi` feature without the `auth` feature

## ~[0.7.7] - 2022-09-15~
### Changed
 - small documentation improvements

## ~[0.7.6] - 2022-09-07~

**This version introduced a compiler error in [#259] when using the `openapi` feature without the `auth` feature enabled.**

### Updated
 - The `gotham_restful_redoc` crate was updated to version 0.2

## [0.7.5] - 2022-07-29
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.74](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc74-2022-07-28)
 - From now on, the redoc handler sits in the `gotham_restful_redoc` crate. The version of `gotham_restful` will not be bumped for every new release of redoc.

## [0.7.4] - 2022-06-02
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.71](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc71-2022-05-31)

## [0.7.3] - 2022-05-13
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.69](https://github.com/Redocly/redoc/releases/tag/2.0.0-rc.69)

## [0.7.2] - 2022-05-12
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.68](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc68-2022-05-10)

## [0.7.1] - 2022-04-29
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.67](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc67-2022-04-28)

## [0.7.0] - 2022-04-07
### Updated
 - `jsonwebtoken` crate to 8.0
 - `openapi_type` crate to 0.4

### Changed
 - When logging errors, the `Debug` implementation is now used (`errorlog` feature only)
 - `AuthStatus::Invalid` now contains the jsonwebtoken error (`auth` feature only)
 - `AuthStatus::Expired` has been removed (`auth` feature only)
 - `AuthError` is now a struct and will include an error message in the payload

## [0.6.7] - 2022-03-31
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.66](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc66-2022-03-30)

## [0.6.6] - 2022-03-16
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.65](https://github.com/Redocly/redoc/releases/tag/v2.0.0-rc.65)

## [0.6.5] - 2022-02-25
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.64](https://github.com/Redocly/redoc/releases/tag/v2.0.0-rc.64)

### Changed
 - rustdoc comments from endpoints are now properly trimmed (`openapi` feature only)

## [0.6.4] - 2022-01-31
### Updated
 - `parking_lot` crate to 0.12

## [0.6.3] - 2022-01-28
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.63](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc63-2022-01-27)
 - The documentation now makes more clear that two types with the same name cause problems (`openapi` feature only)

## [0.6.2] - 2022-01-27
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.62](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc62-2022-01-26)

### Changed
 - Improved some error messages from the derive macros

## [0.6.1] - 2022-01-26
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.61](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc61-2022-01-26)

## [0.6.0] - 2022-01-01
### Updated
 - `openapi_type` crate to 0.3

## [0.5.2] - 2021-12-10
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.59](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc59-2021-12-09)

## [0.5.1] - 2021-11-30
### Updated
 - The linked redoc version has been updated to [2.0.0-rc.58](https://github.com/Redocly/redoc/releases/tag/v2.0.0-rc.58)

## [0.5.0] - 2021-11-13
### Updated
 - gotham to 0.7

## [0.4.6] - 2021-10-15
### Added
 - Support for "dynamic" schema where the endpoint macro creates a new response type for you

### Updated
 - The linked redoc version has been updated to [2.0.0-rc.57](https://github.com/Redocly/redoc/blob/master/CHANGELOG.md#200-rc57-2021-10-11)

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
 [#259]: https://github.com/msrd0/gotham_restful/pull/259
