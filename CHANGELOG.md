# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 13.7.2020
### Changed
 - Changed design of the crate primities, such as `ProxyStream`. It was originally designed that way where a user,
firstly, had to create a connection parameters, and then call a static function `connect` with them to connect through proxy.
This design is inherently much better than classical but awful OOP design where a user has all the unrelated properties,
but NOW, since we have only one method `connect` in the trait `ProxyStream`, it was concluded to remove the connection parameters
and separate a stream building job on a trait `ProxyConstructor` (it is the `ProxyStream` trait, but without `ConnParams` and 
serves for a proxy stream buildig which was used as `ProxyStream`. In future we may back to the former design if we will have
more than one method or other reasons for it.
 - Add Socks5 proxification protocol support (without auth)

## [0.1.1] - 2.7.2020
### Added
 - The first working version of the crate async-proxy has started its routine with the only Socks4 proxification protocol in support
