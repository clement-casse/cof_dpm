//! Module containing some default implementations of the adapters to the Dice Service.

pub mod in_memory;
pub mod noop;

#[cfg(feature = "protobuf")]
pub mod grpc;

#[cfg(feature = "opentelemetry")]
pub mod opentelemetry;

#[cfg(feature = "postgres")]
pub mod postgres;
