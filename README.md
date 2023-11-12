# Prometheus async push

This crate works as an extension to the `prometheus` crate to be able to push non-blocking to your Prometheus push gateway and with a
less dependent setup of `reqwest` (no `openssl` for example). 

By default you have to implement the `Push` trait to use it with your choice of http client or you can use the `with_request` feature. 
This feature implements `Push` in a `PushClient` that leverages `reqwest` under the hood. Reqwest is setup without default features 
(minimal set) so it should not interfere with your reqwest setup (e.g. `rust-tls`).

tbc.
