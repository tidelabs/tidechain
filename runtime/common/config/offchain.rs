// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
  types::{AccountId, BlockHashCount, Index, Signature, SignedPayload, UncheckedExtrinsic},
  Call, Indices, Runtime, System,
};
use codec::Encode;
use sp_runtime::{
  generic,
  traits::{self, SaturatedConversion, StaticLookup},
};

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
  Call: From<LocalCall>,
{
  fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
    call: Call,
    public: <Signature as traits::Verify>::Signer,
    account: AccountId,
    nonce: Index,
  ) -> Option<(
    Call,
    <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
  )> {
    let tip = 0;
    // take the biggest period possible.
    let period = BlockHashCount::get()
      .checked_next_power_of_two()
      .map(|c| c / 2)
      .unwrap_or(2) as u64;
    let current_block = System::block_number()
      .saturated_into::<u64>()
      // The `System::block_number` is initialized with `n+1`,
      // so the actual block number is `n`.
      .saturating_sub(1);
    let extra = (
      frame_system::CheckSpecVersion::<Runtime>::new(),
      frame_system::CheckTxVersion::<Runtime>::new(),
      frame_system::CheckGenesis::<Runtime>::new(),
      frame_system::CheckMortality::<Runtime>::from(generic::Era::mortal(period, current_block)),
      frame_system::CheckNonce::<Runtime>::from(nonce),
      frame_system::CheckWeight::<Runtime>::new(),
      pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
      pallet_tidefi::CheckBytesVectors::<Runtime>::new(),
    );
    let raw_payload = SignedPayload::new(call, extra)
      .map_err(|e| {
        log::warn!("Unable to create signed payload: {:?}", e);
      })
      .ok()?;
    let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
    let address = Indices::unlookup(account);
    let (call, extra, _) = raw_payload.deconstruct();
    Some((call, (address, signature, extra)))
  }
}

impl frame_system::offchain::SigningTypes for Runtime {
  type Public = <Signature as traits::Verify>::Signer;
  type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
  Call: From<C>,
{
  type Extrinsic = UncheckedExtrinsic;
  type OverarchingCall = Call;
}
