use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_collection_rpc_runtime_api::KittyApi as KittyRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait KittyApi<BlockHash> {
	#[rpc(name = "get_launchpad_collections")]
	fn get_launchpad_collections(&self, at: Option<BlockHash>) -> Result<u32>;
}

/// A struct that implements the [`KittyApi`].
pub struct Kitty<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> Kitty<C, P> {
	/// Create new `TransactionPayment` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, Block> KittyApi<<Block as BlockT>::Hash> for Kitty<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: KittyRuntimeApi<Block>,
{
	fn get_launchpad_collections(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result_api = api.get_launchpad_collections(&at);

		result_api.map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to query dispatch info.".into(),
			data: Some(e.to_string().into()),
		})
	}
}
