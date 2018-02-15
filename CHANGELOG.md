# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
### Changed
 - Updated serialport version to 2.1, this cause timeouts to return correct Error.
 - Fixed a bug where pinging protocol 1 servos would always timeout.
 - Fixed a bug where protocol 1 servos would always have model number 29.
### Removed
 - Removed fw version from protocol 1 ServoInfo as it's not contained in protocol 1 pong.
