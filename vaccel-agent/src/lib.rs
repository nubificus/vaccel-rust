#[cfg(not(feature = "async"))]
pub mod rpc_sync;
#[cfg(feature = "async")]
pub mod rpc_async;
