# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
 - Support custom HTTP response headers
 - New `endpoint` router extension with associated `Endpoint` trait ([!18])
 - Support for custom endpoints using the `#[endpoint]` macro ([!19])

### Changed
 - The cors handler can now copy headers from the request if desired
 - All fields of `Response` are now private
 - If not enabling the `openapi` feature, `without-openapi` has to be enabled
 - The endpoint macro attributes (`read`, `create`, ...) no longer take the resource ident and reject all unknown attributes ([!18])

### Removed
 - All pre-defined methods (`read`, `create`, ...) from our router extensions ([!18])
 - All pre-defined method traits (`ResourceRead`, ...) ([!18])

## [0.1.1] - 2020-12-28
### Added
 - Support for `&mut State` parameters in method handlers
 - Support for `NonZeroU` types in the OpenAPI Specification

### Changed
 - cookie auth does not require a middleware for parsing cookies anymore
 - the derive macro produces no more private `mod`s which makes error message more readable
 - documentation now makes use of the `[Type]` syntax introduced in Rust 1.48

## [0.1.0] - 2020-10-02
Previous changes are not tracked by this changelog file. Refer to the [releases](https://gitlab.com/msrd0/gotham-restful/-/releases) for the changelog.


 [!18]: https://gitlab.com/msrd0/gotham-restful/-/merge_requests/18
 [!19]: https://gitlab.com/msrd0/gotham-restful/-/merge_requests/19
