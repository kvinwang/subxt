// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with subxt.  If not, see <http://www.gnu.org/licenses/>.

//! A library to **sub**mit e**xt**rinsics to a
//! [substrate](https://github.com/paritytech/substrate) node via RPC.

use super::{
    SignedPayload,
    UncheckedExtrinsic,
};
use crate::{
    Config,
    ExtrinsicExtraData,
    SignedExtra,
};
use codec::Encode;
use sp_core::Pair;
use sp_runtime::traits::{
    IdentifyAccount,
    SignedExtension,
    Verify,
};

/// Extrinsic signer.
#[async_trait::async_trait]
pub trait Signer<T: Config + ExtrinsicExtraData<T>> {
    /// Returns the account id.
    fn account_id(&self) -> &T::AccountId;

    /// Optionally returns a nonce.
    fn nonce(&self) -> Option<T::Index>;

    /// Takes an unsigned extrinsic and returns a signed extrinsic.
    ///
    /// Some signers may fail, for instance because the hardware on which the keys are located has
    /// refused the operation.
    async fn sign(
        &self,
        extrinsic: SignedPayload<T>,
    ) -> Result<UncheckedExtrinsic<T>, String>;
}

/// Extrinsic signer using a private key.
#[derive(Clone, Debug)]
pub struct PairSigner<T: Config, P: Pair> {
    account_id: T::AccountId,
    nonce: Option<T::Index>,
    signer: P,
}

impl<T, P> PairSigner<T, P>
where
    T: Config + ExtrinsicExtraData<T>,
    T::Signature: From<P::Signature>,
    <T::Signature as Verify>::Signer:
        From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    P: Pair,
{
    /// Creates a new `Signer` from a `Pair`.
    pub fn new(signer: P) -> Self {
        let account_id =
            <T::Signature as Verify>::Signer::from(signer.public()).into_account();
        Self {
            account_id,
            nonce: None,
            signer,
        }
    }

    /// Sets the nonce to a new value.
    pub fn set_nonce(&mut self, nonce: T::Index) {
        self.nonce = Some(nonce);
    }

    /// Increment the nonce.
    pub fn increment_nonce(&mut self) {
        self.nonce = self.nonce.map(|nonce| nonce + 1u32.into());
    }

    /// Returns the signer.
    pub fn signer(&self) -> &P {
        &self.signer
    }
}

#[async_trait::async_trait]
impl<T, P> Signer<T> for PairSigner<T, P>
where
    T: Config + ExtrinsicExtraData<T>,
    T::AccountId: Into<T::Address> + 'static,
    <<<T as ExtrinsicExtraData<T>>::Extra as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned: Send + Sync + 'static,
    P: Pair + 'static,
    P::Signature: Into<T::Signature> + 'static,
{
    fn account_id(&self) -> &T::AccountId {
        &self.account_id
    }

    fn nonce(&self) -> Option<T::Index> {
        self.nonce
    }

    async fn sign(
        &self,
        extrinsic: SignedPayload<T>,
    ) -> Result<UncheckedExtrinsic<T>, String> {
        let signature = extrinsic.using_encoded(|payload| self.signer.sign(payload));
        let (call, extra, _) = extrinsic.deconstruct();
        let extrinsic = UncheckedExtrinsic::<T>::new_signed(
            call,
            self.account_id.clone().into(),
            signature.into(),
            extra,
        );
        Ok(extrinsic)
    }
}
