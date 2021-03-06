// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Strongly typed API for light Polkadot client.

use std::sync::Arc;
use client::backend::{Backend, RemoteBackend};
use client::{Client, CallExecutor};
use codec::Slicable;
use state_machine;
use primitives::{AccountId, Block, BlockId, Hash, Index, SessionKey, Timestamp, UncheckedExtrinsic};
use primitives::parachain::{CandidateReceipt, DutyRoster, Id as ParaId};
use full::CheckedId;
use {PolkadotApi, BlockBuilder, RemotePolkadotApi, CheckedBlockId, Result, ErrorKind};

/// Light block builder. TODO: make this work (efficiently)
#[derive(Clone, Copy)]
pub struct LightBlockBuilder;

impl BlockBuilder for LightBlockBuilder {
	fn push_extrinsic(&mut self, _xt: UncheckedExtrinsic) -> Result<()> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn bake(self) -> Result<Block> {
		Err(ErrorKind::UnknownRuntime.into())
	}
}

/// Remote polkadot API implementation.
pub struct RemotePolkadotApiWrapper<B: Backend<Block>, E: CallExecutor<Block>>(pub Arc<Client<B, E, Block>>);

impl<B: Backend<Block>, E: CallExecutor<Block>> PolkadotApi for RemotePolkadotApiWrapper<B, E>
	where ::client::error::Error: From<<<B as Backend<Block>>::State as state_machine::backend::Backend>::Error>
{
	type CheckedBlockId = CheckedId;
	type BlockBuilder = LightBlockBuilder;

	fn check_id(&self, id: BlockId) -> Result<CheckedId> {
		Ok(CheckedId(id))
	}

	fn session_keys(&self, at: &CheckedId) -> Result<Vec<SessionKey>> {
		self.0.executor().call(at.block_id(), "authorities", &[])
			.and_then(|r| Vec::<SessionKey>::decode(&mut &r.return_data[..])
				.ok_or("error decoding session keys".into()))
			.map_err(Into::into)
	}

	fn validators(&self, _at: &CheckedId) -> Result<Vec<AccountId>> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn random_seed(&self, _at: &Self::CheckedBlockId) -> Result<Hash> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn duty_roster(&self, _at: &CheckedId) -> Result<DutyRoster> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn timestamp(&self, _at: &CheckedId) -> Result<Timestamp> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn evaluate_block(&self, _at: &CheckedId, _block: Block) -> Result<bool> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn index(&self, _at: &CheckedId, _account: AccountId) -> Result<Index> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn active_parachains(&self, _at: &Self::CheckedBlockId) -> Result<Vec<ParaId>> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn parachain_code(&self, _at: &Self::CheckedBlockId, _parachain: ParaId) -> Result<Option<Vec<u8>>> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn parachain_head(&self, _at: &Self::CheckedBlockId, _parachain: ParaId) -> Result<Option<Vec<u8>>> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn build_block(&self, _at: &Self::CheckedBlockId, _timestamp: Timestamp, _new_heads: Vec<CandidateReceipt>) -> Result<Self::BlockBuilder> {
		Err(ErrorKind::UnknownRuntime.into())
	}

	fn inherent_extrinsics(&self, _at: &Self::CheckedBlockId, _timestamp: Timestamp, _new_heads: Vec<CandidateReceipt>) -> Result<Vec<Vec<u8>>> {
		Err(ErrorKind::UnknownRuntime.into())
	}
}

impl<B: RemoteBackend<Block>, E: CallExecutor<Block>> RemotePolkadotApi for RemotePolkadotApiWrapper<B, E>
	where ::client::error::Error: From<<<B as Backend<Block>>::State as state_machine::backend::Backend>::Error>
{}
