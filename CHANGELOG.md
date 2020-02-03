# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
