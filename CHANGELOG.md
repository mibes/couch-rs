# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/mibes/couch-rs/compare/0.9.2...develop) - ReleaseDate

## [0.9.2] - 2023-03-10

### Changed

- Fixed RUSTSEC-2023-0018; updated base64 to 0.21; contributed by tkoeppen.

## [0.9.1] - 2023-01-03

### Added

- Add `delete_index` operation; contributed by kingledion

### Changed
- Update the arguments to `insert_index` to match the `types::index::Index struct`, to make it more compatible with the `get_indexes` method; contributed by kingledion
- DEPRECATED: `ensure_index`

## [0.9.0] - 2022-08-23

### Changed

- BREAKING change: `CouchError` is now an enum to easily differentiate using a match; contributed by jgrund
- Add `membership` and `cluster_setup` read operations; contributed by hmacias

## [0.8.39] - 2022-07-13

### Changed

- Dev dependencies updated for tokio; contributed by tkoeppen

## [0.8.38] - 2022-07-12

### Changed

- Set minimal tokio version to 1.14 to ensure RUSTSEC-2021-0124 is patched.
- Split unit-, doc- and integration-tests; contributed by tkoeppen
- Percent encode the database name to accommodate special characters like '+'.

## [0.8.37] - 2022-03-25

### Changed

- Allow non-string keys for views; contributed by marius851000
- Updated tokio-util to 0.7

## [0.8.36] - 2022-02-09

### Added

- Added into_option() to turn a Result into a Result<Option> to easily handle non-existing documents (404)

## [0.8.35] - 2022-01-31

### Changed

- Fixed is_ok() to accept HTTP status 200 <= n < 300 as successes too

## [0.8.34] - 2021-11-29

### Changed

- Reverted to 2018 edition (address issue: [#17](https://github.com/mibes/couch-rs/issues/17))

## [0.8.33] - 2021-11-11

### Changed

- 2021 edition
- Clippy suggestions fixed
- BUG FIX: client.exists() returns true too often; contributed by wrazik & horacimacias


## [0.8.32] - 2021-09-08

### Changed

- allow to specify TLS flavour; contributed by sify21
- BUG FIX: use `serde_json::Value` for `seq` when querying `_changes`

## [0.8.31] - 2021-07-20

### Added

- stream changes; contributed by Frando
- bulk upsert; contributed by Frando

## [0.8.30] - 2021-04-06

### Changed

- cleaned-up some left-over dbg! statements

## [0.8.29] - 2021-03-30

### Changed

- BREAKING CHANGE: mutually borrow documents on create and update; contributed by horacimacias

## [0.8.28] - 2021-02-05

### Changed

- Loosened the dependency requirements

## [0.8.27] - 2021-01-14

### Changed

- Depend on std::fmt::Formatter not the serde::export one.

## [0.8.26] - 2021-01-06

### Changed

- Upgraded reqwest to 0.11 and tokio to 1.0

### Added

- Add support for partitioned databases; contibuted by krishna-kashyap

## [0.8.25] - 2020-11-16

### Changed

- Ditched a `unwrap`.

## [0.8.24] - 2020-10-06

### Changed

- Renamed `merge` to `merge_ids` to avoid confusion.
- Allow `CouchError` to include an optional `id`

## [0.8.23] - 2020-10-05

### Changed

- Use `unwrap_or_default` when extracting a json field, prevents panic when finding documents without an `_id`
  field in the result.

## [0.8.22] - 2020-10-01

### Changed

- Typed view queries

## [0.8.21] - 2020-10-01

### Changed

- Allow `_bulk_docs` to take TypedDocuments

## [0.8.19] - 2020-10-01

### Changed

- WARNING: big changes ahead!
- Most of the find/get operations now take a typed `TypedCouchDocument`.
  - To use the generic `Value`, either use a `.._raw` function, or type the query with `::<Value>`
  - Value now holds the raw CouchDocument, including `_id` and `_rev` fields.
  - See the examples and the tests for more details
- `TypedCouchDocument` traits can be derived using `CouchDocument`
  - `Value` implements `TypedCouchDocument` traits

## [0.8.18] - 2020-09-29

### Changed

- Allow complex keys on views
- Url encode additional ID's

## [0.8.17] - 2020-09-29

### Changed

- Url encoding to allow for complex IDs, for example `1+2`

### Added

- Get database information

## [0.8.16] - 2020-09-28

### Changed

- BUG FIX: find_batched should have sent segment_query, but was sending query

## [0.8.15] - 2020-09-28

### Changed

- Use Basic Authentication headers

## [0.8.14] - 2020-09-28

### Changed

- bulk_docs returns a Vec of CouchResults
- Updated examples

## [0.8.13] - 2020-09-28

### Added

- Included builder paradigm for FindQuery and QueryParams

## [0.8.12] - 2020-09-27

### Changed

- Use `&str` instead of `String` in a few places to make the API easier to use
- Included test for `bulk_docs`

## [0.8.11] - 2020-09-27

### Changed

- Use `&str` instead of `DocumentId` in a few places to make the API easier to use
- Include an example for `query_many_all_docs`

## [0.8.10] - 2020-09-27

### Changed

- Introduce gitflow
- Updated rustfmt configuration
- Test for query params

## [0.8.9] - 2020-09-27

### Changed

- Let `get_bulk_params` take `Option<QueryParams>`

## [0.8.8] - 2020-09-27

### Changed

- Use `Into<serde_json::Value>` trait for `create_view` to not break compatibility with `CouchUpdate`

## [0.8.7] - 2020-09-27

### Changed

- Use the typed `CouchViews` structure to create views

## [0.8.6] - 2020-09-27

### Added

- Included the `upsert` operation

## [0.8.5] - 2020-09-26

### Added

- Implement multiple queries in a single request

## [0.8.4] - 2020-09-26

### Changed

- Updated the documentation and examples

## [0.8.3] - 2020-09-26

### Changed

- Automated GitHub build action

## [0.8.0] - 2020-09-26

### Changed

- the `find()` operations takes a FindQuery

## [0.7.18] - 2020-09-25

### Changed

- Views now use QueryParams instead of an untyped map.
- Views can now return the optional "doc" item.
- BREAKING CHANGE: `execute_view` has been removed. Use `query` instead.

## [0.7.17] - 2020-09-14

### Changed

- Sort takes an array of key/value pairs, like: [{"first_name":"desc"}]

## [0.7.16] - 2020-09-14

### Changed

- Make total_rows in ViewCollection optional.

## [0.7.15] - 2020-09-14

### Changed

- Make id in ViewItem optional.

## [0.7.14] - 2020-09-14

### Changed

- Return value in ViewItem as a Value, not String

## [0.7.13] - 2020-09-11

### Changed

- Use reqwest's `error_for_status()` on responses, where we are not actively checking the result.
- Return an Error when one occurs during batch reading.
- Removed the `'static` lifetime on some of the `str` parameters; contribution from kallisti5
- Included `execute_update()` operation; contribution from horacimacias

## [0.7.12] - 2020-09-10

### Changed

- Check response success for create_view()

## [0.7.11] - 2020-09-09

### Changed

- Allow to query a view with a different design name

## [0.7.10] - 2020-09-09

### Changed

- BREAKING CHANGE: get_all_params now takes a typed QueryParams as input.
- get_all_params uses POST, instead of GET, for greater flexibility.

## [0.7.9] - 2020-09-09

### Changed

- `json_extr!` does not panic when called on a non-existent field. Like in find for \_id, when the find result does not
  include an \_id.

## [0.7.8] - 2020-09-09

### Changed

- Implemented Display for FindQuery

## [0.7.7] - 2020-09-09

### Changed

- Allow FindQuery to be converted to Value

## [0.7.6] - 2020-09-09

### Added

- Added `find_batched` to allow asynchronous customized searches

## [0.7.0] - 2020-02-03

### Added

- Added `Client::make_db`
- Added `docker-compose.yml`
- Added `.rustfmt.toml`

### Changed

- Updated to the Rust 2018 edition standards
- Compiles against the latest reqwest and serde libraries
- Optimized memory consumption by moving `iter()` calls to `into_iter()` where needed
- Changed `SofaError` to derive `failure`
- Changed `Client::check_status` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::create_path` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::db` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::delete` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::destroy_db` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::get` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::gzip` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::head` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::list_dbs` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::new` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::post` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::pub` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::req` signature to remove potentially panicking `unwrap()` calls
- Changed `Client::timeout` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::create` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::ensure_index` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::find` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::get` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::insert_index` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::read_indexes` signature to remove potentially panicking `unwrap()` calls
- Changed `Database::save` signature to remove potentially panicking `unwrap()` calls

### Removed

- Removed env files that were necessary for single-threaded test run. Added section in README to reflect that.
- Removed the `failure` dependency
