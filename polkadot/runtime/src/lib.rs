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

//! The Polkadot runtime. This can be compiled with ``#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "std")]
extern crate serde;

#[macro_use]
extern crate substrate_runtime_io as runtime_io;

#[macro_use]
extern crate substrate_runtime_support;

#[macro_use]
extern crate substrate_runtime_primitives as runtime_primitives;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

#[cfg(test)]
extern crate substrate_serializer;

#[cfg_attr(feature = "std", macro_use)]
extern crate substrate_primitives;

#[macro_use]
extern crate substrate_runtime_std as rstd;

extern crate polkadot_primitives as primitives;
extern crate substrate_codec as codec;
extern crate substrate_runtime_consensus as consensus;
extern crate substrate_runtime_council as council;
extern crate substrate_runtime_democracy as democracy;
extern crate substrate_runtime_executive as executive;
extern crate substrate_runtime_session as session;
extern crate substrate_runtime_staking as staking;
extern crate substrate_runtime_system as system;
extern crate substrate_runtime_timestamp as timestamp;

mod parachains;

use rstd::prelude::*;
use primitives::{AccountId, Balance, BlockNumber, Hash, Index, Log, SessionKey, Signature};
use primitives::parachain::CandidateReceipt;
use runtime_primitives::{generic, traits::{HasPublicAux, BlakeTwo256, Convert}};

#[cfg(feature = "std")]
pub use runtime_primitives::BuildExternalities;

pub use consensus::Call as ConsensusCall;
pub use timestamp::Call as TimestampCall;
pub use parachains::Call as ParachainsCall;
pub use primitives::Header;

/// The position of the timestamp set extrinsic.
pub const TIMESTAMP_SET_POSITION: u32 = 0;
/// The position of the parachains set extrinsic.
pub const PARACHAINS_SET_POSITION: u32 = 1;

/// Block Id type for this block.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, Index, Call, Signature>;
/// Extrinsic type as expected by this runtime.
pub type Extrinsic = generic::Extrinsic<AccountId, Index, Call>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Provides a type-safe wrapper around a structurally valid block.
#[cfg(feature = "std")]
pub struct CheckedBlock {
	inner: Block,
	file_line: Option<(&'static str, u32)>,
}

#[cfg(feature = "std")]
impl CheckedBlock {
	/// Create a new checked block. Fails if the block is not structurally valid.
	pub fn new(block: Block) -> Result<Self, Block> {
		let has_timestamp = block.extrinsics.get(TIMESTAMP_SET_POSITION as usize).map_or(false, |xt| {
			!xt.is_signed() && match xt.extrinsic.function {
				Call::Timestamp(TimestampCall::set(_)) => true,
				_ => false,
			}
		});

		if !has_timestamp { return Err(block) }

		let has_heads = block.extrinsics.get(PARACHAINS_SET_POSITION as usize).map_or(false, |xt| {
			!xt.is_signed() && match xt.extrinsic.function {
				Call::Parachains(ParachainsCall::set_heads(_)) => true,
				_ => false,
			}
		});

		if !has_heads { return Err(block) }
		Ok(CheckedBlock {
			inner: block,
			file_line: None,
		})
	}

	// Creates a new checked block, asserting that it is valid.
	#[doc(hidden)]
	pub fn new_unchecked(block: Block, file: &'static str, line: u32) -> Self {
		CheckedBlock {
			inner: block,
			file_line: Some((file, line)),
		}
	}

	/// Extract the timestamp from the block.
	pub fn timestamp(&self) -> ::primitives::Timestamp {
		let x = self.inner.extrinsics.get(TIMESTAMP_SET_POSITION as usize).and_then(|xt| match xt.extrinsic.function {
			Call::Timestamp(TimestampCall::set(x)) => Some(x),
			_ => None
		});

		match x {
			Some(x) => x,
			None => panic!("Invalid polkadot block asserted at {:?}", self.file_line),
		}
	}

	/// Extract the parachain heads from the block.
	pub fn parachain_heads(&self) -> &[CandidateReceipt] {
		let x = self.inner.extrinsics.get(PARACHAINS_SET_POSITION as usize).and_then(|xt| match xt.extrinsic.function {
			Call::Parachains(ParachainsCall::set_heads(ref x)) => Some(&x[..]),
			_ => None
		});

		match x {
			Some(x) => x,
			None => panic!("Invalid polkadot block asserted at {:?}", self.file_line),
		}
	}

	/// Convert into inner block.
	pub fn into_inner(self) -> Block { self.inner }
}

#[cfg(feature = "std")]
impl ::std::ops::Deref for CheckedBlock {
	type Target = Block;

	fn deref(&self) -> &Block { &self.inner }
}

/// Assert that a block is structurally valid. May lead to panic in the future
/// in case it isn't.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! assert_polkadot_block {
	($block: expr) => {
		$crate::CheckedBlock::new_unchecked($block, file!(), line!())
	}
}

/// Concrete runtime type used to parameterize the various modules.
pub struct Concrete;

impl HasPublicAux for Concrete {
	type PublicAux = AccountId;	// TODO: Option<AccountId>
}

impl system::Trait for Concrete {
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Digest = generic::Digest<Log>;
	type AccountId = AccountId;
	type Header = Header;
}
/// System module for this concrete runtime.
pub type System = system::Module<Concrete>;

impl consensus::Trait for Concrete {
	type PublicAux = <Concrete as HasPublicAux>::PublicAux;
	type SessionKey = SessionKey;
}
/// Consensus module for this concrete runtime.
pub type Consensus = consensus::Module<Concrete>;

impl timestamp::Trait for Concrete {
	const SET_POSITION: u32 = TIMESTAMP_SET_POSITION;
	type Value = u64;
}
/// Timestamp module for this concrete runtime.
pub type Timestamp = timestamp::Module<Concrete>;

/// Session key conversion.
pub struct SessionKeyConversion;
impl Convert<AccountId, SessionKey> for SessionKeyConversion {
	fn convert(a: AccountId) -> SessionKey {
		a.0
	}
}

impl session::Trait for Concrete {
	type ConvertAccountIdToSessionKey = SessionKeyConversion;
}
/// Session module for this concrete runtime.
pub type Session = session::Module<Concrete>;

impl staking::Trait for Concrete {
	type Balance = Balance;
	type DetermineContractAddress = BlakeTwo256;
}
/// Staking module for this concrete runtime.
pub type Staking = staking::Module<Concrete>;

impl democracy::Trait for Concrete {
	type Proposal = PrivCall;
}
/// Democracy module for this concrete runtime.
pub type Democracy = democracy::Module<Concrete>;

impl council::Trait for Concrete {}
/// Council module for this concrete runtime.
pub type Council = council::Module<Concrete>;
/// Council voting module for this concrete runtime.
pub type CouncilVoting = council::voting::Module<Concrete>;

impl parachains::Trait for Concrete {
	const SET_POSITION: u32 = PARACHAINS_SET_POSITION;

	type PublicAux = <Concrete as HasPublicAux>::PublicAux;
}
pub type Parachains = parachains::Module<Concrete>;

impl_outer_dispatch! {
	/// Call type for polkadot transactions.
	#[derive(Clone, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
	pub enum Call where aux: <Concrete as HasPublicAux>::PublicAux {
		Consensus = 0,
		Session = 1,
		Staking = 2,
		Timestamp = 3,
		Democracy = 5,
		Council = 6,
		CouncilVoting = 7,
		Parachains = 8,
	}

	/// Internal calls.
	#[derive(Clone, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
	pub enum PrivCall {
		Consensus = 0,
		Session = 1,
		Staking = 2,
		Democracy = 5,
		Council = 6,
		CouncilVoting = 7,
	}
}

/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Concrete, Block, Staking,
	(((((((), Parachains), Council), Democracy), Staking), Session), Timestamp)>;

impl_outer_config! {
	pub struct GenesisConfig for Concrete {
		ConsensusConfig => consensus,
		SystemConfig => system,
		SessionConfig => session,
		StakingConfig => staking,
		DemocracyConfig => democracy,
		CouncilConfig => council,
		ParachainsConfig => parachains,
	}
}

/// Produces the list of inherent extrinsics.
pub fn inherent_extrinsics(timestamp: ::primitives::Timestamp, parachain_heads: Vec<CandidateReceipt>) -> Vec<UncheckedExtrinsic> {
	vec![
		UncheckedExtrinsic {
			extrinsic: Extrinsic {
				signed: Default::default(),
				function: Call::Timestamp(TimestampCall::set(timestamp)),
				index: 0,
			},
			signature: Default::default(),
		},
		UncheckedExtrinsic {
			extrinsic: Extrinsic {
				signed: Default::default(),
				function: Call::Parachains(ParachainsCall::set_heads(parachain_heads)),
				index: 0,
			},
			signature: Default::default(),
		},
	]
}

/// Checks an unchecked extrinsic for validity.
pub fn check_extrinsic(xt: UncheckedExtrinsic) -> bool {
	use runtime_primitives::traits::Checkable;

	xt.check().is_ok()
}

pub mod api {
	impl_stubs!(
		authorities => |()| super::Consensus::authorities(),
		initialise_block => |header| super::Executive::initialise_block(&header),
		apply_extrinsic => |extrinsic| super::Executive::apply_extrinsic(extrinsic),
		execute_block => |block| super::Executive::execute_block(block),
		finalise_block => |()| super::Executive::finalise_block(),
		inherent_extrinsics => |(timestamp, heads)| super::inherent_extrinsics(timestamp, heads),
		validator_count => |()| super::Session::validator_count(),
		validators => |()| super::Session::validators()
	);
}

#[cfg(test)]
mod tests {
	use super::*;
	use substrate_primitives as primitives;
	use ::codec::Slicable;
	use substrate_primitives::hexdisplay::HexDisplay;
	use substrate_serializer as ser;
	use runtime_primitives::traits::{Digest as DigestT, Header as HeaderT};
	type Digest = generic::Digest<Log>;

	#[test]
	fn test_header_serialization() {
		let header = Header {
			parent_hash: 5.into(),
			number: 67,
			state_root: 3.into(),
			extrinsics_root: 6.into(),
			digest: { let mut d = Digest::default(); d.push(Log(vec![1])); d },
		};

		assert_eq!(ser::to_string_pretty(&header), r#"{
  "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000005",
  "number": 67,
  "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000003",
  "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000006",
  "digest": {
    "logs": [
      "0x01"
    ]
  }
}"#);

		let v = header.encode();
		assert_eq!(Header::decode(&mut &v[..]).unwrap(), header);
	}

	#[test]
	fn block_encoding_round_trip() {
		let mut block = Block {
			header: Header::new(1, Default::default(), Default::default(), Default::default(), Default::default()),
			extrinsics: vec![
				UncheckedExtrinsic {
					extrinsic: Extrinsic {
						function: Call::Timestamp(timestamp::Call::set(100_000_000)),
						signed: Default::default(),
						index: Default::default(),
					},
					signature: Default::default(),
				}
			],
		};

		let raw = block.encode();
		let decoded = Block::decode(&mut &raw[..]).unwrap();

		assert_eq!(block, decoded);

		block.extrinsics.push(UncheckedExtrinsic {
			extrinsic: Extrinsic {
				function: Call::Staking(staking::Call::stake()),
				signed: Default::default(),
				index: 10101,
			},
			signature: Default::default(),
		});

		let raw = block.encode();
		let decoded = Block::decode(&mut &raw[..]).unwrap();

		assert_eq!(block, decoded);
	}

	#[test]
	fn block_encoding_substrate_round_trip() {
		let mut block = Block {
			header: Header::new(1, Default::default(), Default::default(), Default::default(), Default::default()),
			extrinsics: vec![
				UncheckedExtrinsic {
					extrinsic: Extrinsic {
						function: Call::Timestamp(timestamp::Call::set(100_000_000)),
						signed: Default::default(),
						index: Default::default(),
					},
					signature: Default::default(),
				}
			],
		};

		block.extrinsics.push(UncheckedExtrinsic {
			extrinsic: Extrinsic {
				function: Call::Staking(staking::Call::stake()),
				signed: Default::default(),
				index: 10101,
			},
			signature: Default::default(),
		});

		let raw = block.encode();
		let decoded_primitive = ::primitives::Block::decode(&mut &raw[..]).unwrap();
		let encoded_primitive = decoded_primitive.encode();
		let decoded = Block::decode(&mut &encoded_primitive[..]).unwrap();

		assert_eq!(block, decoded);
	}

	#[test]
	fn serialize_unchecked() {
		let tx = UncheckedExtrinsic {
			extrinsic: Extrinsic {
				signed: [1; 32].into(),
				index: 999,
				function: Call::Timestamp(TimestampCall::set(135135)),
			},
			signature: runtime_primitives::Ed25519Signature(primitives::hash::H512([0; 64])).into(),
		};
		// 71000000
		// 0101010101010101010101010101010101010101010101010101010101010101
		// e703000000000000
		// 00
		// df0f0200
		// 0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000

		let v = Slicable::encode(&tx);
		println!("{}", HexDisplay::from(&v));
		assert_eq!(UncheckedExtrinsic::decode(&mut &v[..]).unwrap(), tx);
	}

	#[test]
	fn serialize_checked() {
		let xt = Extrinsic {
			signed: hex!["0d71d1a9cad6f2ab773435a7dec1bac019994d05d1dd5eb3108211dcf25c9d1e"].into(),
			index: 0,
			function: Call::CouncilVoting(council::voting::Call::propose(Box::new(
				PrivCall::Consensus(consensus::PrivCall::set_code(
					vec![]
				))
			))),
		};
		let v = Slicable::encode(&xt);

		let data = hex!["e00000000d71d1a9cad6f2ab773435a7dec1bac019994d05d1dd5eb3108211dcf25c9d1e0000000007000000000000006369D39D892B7B87A6769F90E14C618C2B84EBB293E2CC46640136E112C078C75619AC2E0815F2511568736623C055156C8FC427CE2AEE4AE2838F86EFE80208"];
		let uxt: UncheckedExtrinsic = Slicable::decode(&mut &data[..]).unwrap();
		assert_eq!(uxt.extrinsic, xt);

		assert_eq!(Extrinsic::decode(&mut &v[..]).unwrap(), xt);
	}
}
