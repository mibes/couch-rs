# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2020-09-26

- the `find()` operations takes a FindQuery

## [0.7.18] - 2020-09-25

- Views now use QueryParams instead of an untyped map.
- Views can now return the optional "doc" item.
- BREAKING CHANGE: `execute_view` has been removed. Use `query` instead.

## [0.7.17] - 2020-09-14

- Sort takes an array of key/value pairs, like: [{"first_name":"desc"}]

## [0.7.16] - 2020-09-14

- Make total_rows in ViewCollection optional.

## [0.7.15] - 2020-09-14

- Make id in ViewItem optional.

## [0.7.14] - 2020-09-14
- Return value in ViewItem as a Value, not String

## [0.7.13] - 2020-09-11
- Use reqwest's `error_for_status()` on responses, where we are not actively checking the result.
- Return an Error when one occurs during batch reading.
- Removed the `'static` lifetime on some of the `str` parameters; contribution from kallisti5
- Included `execute_update()` operation; contribution from horacimacias 

## [0.7.12] - 2020-09-10
- Check response success for create_view()

## [0.7.11] - 2020-09-09
- Allow to query a view with a different design name

## [0.7.10] - 2020-09-09
- BREAKING CHANGE: get_all_params now takes a typed QueryParams as input.
- get_all_params uses POST, instead of GET, for greater flexibility.

## [0.7.9] - 2020-09-09
- `json_extr!` does not panic when called on a non-existent field. Like in find for _id, 
   when the find result does not include an _id.

## [0.7.8] - 2020-09-09
- Implemented Display for FindQuery

## [0.7.7] - 2020-09-09
- Allow FindQuery to be converted to Value

## [0.7.6] - 2020-09-09
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
