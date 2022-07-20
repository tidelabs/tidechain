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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub(crate) const LOG_TARGET: &str = "tidefi::quorum";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), T::Security::get_current_block_count() $(, $values)*
		)
	};
}

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    log,
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
  };
  use frame_system::pallet_prelude::*;
  use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
  };
  use sp_std::{vec, vec::Vec};
  use tidefi_primitives::{
    assets::Asset,
    pallet::{AssetRegistryExt, QuorumExt, SecurityExt, SunriseExt},
    AssetId, Balance, ComplianceLevel, CurrencyId, Hash, Mint, ProposalStatus, ProposalType,
    ProposalVotes, WatchList, WatchListAction, Withdrawal,
  };

  #[pallet::config]
  /// Configure the pallet by specifying the parameters and types on which it depends.
  pub trait Config:
    frame_system::Config + pallet_assets::Config<AssetId = AssetId, Balance = Balance>
  {
    /// Events
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    /// Pallet ID
    #[pallet::constant]
    type QuorumPalletId: Get<PalletId>;

    /// Proposals capacity
    #[pallet::constant]
    type ProposalsCap: Get<u32>;

    /// Burned queue capacity
    #[pallet::constant]
    type BurnedCap: Get<u32>;

    /// Proposals lifetime
    #[pallet::constant]
    type ProposalLifetime: Get<Self::BlockNumber>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

    /// Sunrise traits
    type Sunrise: SunriseExt<Self::AccountId, Self::BlockNumber>;

    /// The maximum length of string (public keys etc..)
    #[pallet::constant]
    type StringLimit: Get<u32>;

    /// The maximum number of votes per proposal
    #[pallet::constant]
    type VotesLimit: Get<u32>;

    /// The maximum number of proposals per account watch list
    #[pallet::constant]
    type WatchListLimit: Get<u32>;

    /// The pubkey per asset (should always be more than current member size)
    #[pallet::constant]
    type PubkeyLimitPerAsset: Get<u32>;

    /// Asset registry traits
    type AssetRegistry: AssetRegistryExt;

    /// Tidechain currency wrapper
    type CurrencyTidefi: Inspect<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Mutate<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
      + Transfer<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub (super) trait Store)]
  pub struct Pallet<T>(_);

  /// Quorum status
  #[pallet::storage]
  #[pallet::getter(fn status)]
  pub(super) type QuorumStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

  /// Quorum public keys for all chains
  #[pallet::storage]
  #[pallet::getter(fn public_keys)]
  pub type PublicKeys<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    AssetId,
    BoundedVec<
      (
        T::AccountId,
        BoundedVec<u8, <T as pallet::Config>::StringLimit>,
      ),
      T::PubkeyLimitPerAsset,
    >,
    ValueQuery,
  >;

  /// Set of active transaction to watch
  #[pallet::storage]
  #[pallet::getter(fn account_watch_list)]
  pub type AccountWatchList<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<
      WatchList<T::BlockNumber, BoundedVec<u8, <T as pallet::Config>::StringLimit>>,
      <T as pallet::Config>::WatchListLimit,
    >,
  >;

  /// The threshold required for a proposal to process
  #[pallet::storage]
  #[pallet::getter(fn threshold)]
  pub type Threshold<T: Config> = StorageValue<_, u16, ValueQuery>;

  /// Set of proposals for the Quorum
  #[pallet::storage]
  #[pallet::getter(fn proposals)]
  pub type Proposals<T: Config> = StorageValue<
    _,
    BoundedVec<
      (
        Hash,
        T::BlockNumber,
        ProposalType<
          T::AccountId,
          T::BlockNumber,
          BoundedVec<u8, <T as pallet::Config>::StringLimit>,
          BoundedVec<T::AccountId, <T as pallet::Config>::VotesLimit>,
        >,
      ),
      T::ProposalsCap,
    >,
    ValueQuery,
  >;

  /// Set of Votes for each proposal
  #[pallet::storage]
  #[pallet::getter(fn proposal_votes)]
  pub type Votes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Hash,
    ProposalVotes<T::BlockNumber, BoundedVec<T::AccountId, T::VotesLimit>>,
  >;

  /// Set of active quorum members
  #[pallet::storage]
  #[pallet::getter(fn members)]
  pub type Members<T: Config> = CountedStorageMap<_, Blake2_128Concat, T::AccountId, bool>;

  /// Burned queue
  #[pallet::storage]
  #[pallet::getter(fn burned_queue)]
  pub type BurnedQueue<T: Config> = StorageValue<
    _,
    BoundedVec<
      (
        Hash,
        Withdrawal<
          T::AccountId,
          T::BlockNumber,
          BoundedVec<u8, <T as pallet::Config>::StringLimit>,
        >,
      ),
      T::BurnedCap,
    >,
    ValueQuery,
  >;

  /// Genesis configuration
  #[pallet::genesis_config]
  pub struct GenesisConfig<T: Config> {
    /// Quorum status
    pub enabled: bool,
    /// Quorum members
    pub members: Vec<T::AccountId>,
    /// Quorum threshold to process a proposal
    pub threshold: u16,
  }

  #[cfg(feature = "std")]
  impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
      Self {
        // Quorum is enabled by default
        enabled: true,
        members: Vec::new(),
        threshold: 1,
      }
    }
  }

  #[pallet::genesis_build]
  impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
      Threshold::<T>::put(self.threshold);
      QuorumStatus::<T>::put(self.enabled);
      for account_id in &self.members {
        Members::<T>::insert(account_id, true);
      }
    }
  }

  #[pallet::event]
  #[pallet::generate_deposit(pub (super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Quorum status changed
    StatusChanged { is_enabled: bool },
    /// Quorum account changed
    AccountChanged { account_id: T::AccountId },
    /// Quorum minted token to the account
    Minted {
      proposal_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      transaction_id: Vec<u8>,
      compliance_level: ComplianceLevel,
    },
    /// A new transaction has been added to the watch list
    WatchTransactionAdded {
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      compliance_level: ComplianceLevel,
      transaction_id: Vec<u8>,
      watch_action: WatchListAction,
    },

    /// Quorum burned token to the account from tidechain
    BurnedInitialized {
      proposal_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },

    /// Quorum member acknowledged the burned and initiated the process
    BurnedAcknowledged { proposal_id: Hash },

    /// Member voted for a proposal
    VoteFor {
      account_id: T::AccountId,
      proposal_id: Hash,
    },

    /// Member voted against a proposal
    VoteAgainst {
      account_id: T::AccountId,
      proposal_id: Hash,
    },

    /// Proposal has been processed successfully
    ProposalSubmitted { proposal_id: Hash },

    /// Proposal has been approved
    ProposalApproved { proposal_id: Hash },

    /// Proposal has been processed successfully
    ProposalProcessed { proposal_id: Hash },

    /// Proposal has been rejected
    ProposalRejected { proposal_id: Hash },

    /// The quorum configuration has been updated, all elected members should re-submit public keys
    ConfigurationUpdated {
      members: Vec<T::AccountId>,
      threshold: u16,
    },
  }

  // Errors inform users that something went wrong.
  #[pallet::error]
  pub enum Error<T> {
    /// Asset is currently disabled or do not exist on chain
    AssetDisabled,
    /// The access to the Quorum pallet is not allowed for this account ID.
    AccessDenied,
    /// Unable to burn token.
    BurnFailed,
    /// Proposals cap exceeded, try again later.
    ProposalsCapExceeded,
    /// No proposal with the ID was found
    ProposalDoesNotExist,
    /// Proposal block number is in the future.
    ProposalBlockIsInFuture,
    /// Proposal has either failed or succeeded
    ProposalAlreadyComplete,
    /// Lifetime of proposal has been exceeded
    ProposalExpired,
    /// Member already voted for this proposal
    MemberAlreadyVoted,
    /// Mint failed
    MintFailed,
    /// Invalid proposal
    BadProposal,
    /// Invalid public key
    BadPublicKey,
    /// Invalid transaction id
    BadTransactionId,
    /// Invalid external address
    BadExternalAddress,
    /// Burned queue cap reached
    BurnedQueueOverflow,
    /// Watchlist cap reached
    WatchlistOverflow,
    /// Members cap reached
    MembersOverflow,
    /// Votes for cap reached for this proposal
    VotesForOverflow,
    /// Votes against cap reached for this proposal
    VotesAgainstOverflow,
    /// Public keys cap reached for this asset id
    PublicKeysOverflow,
    // Unknown error
    UnknownError,
    /// Invalid asset
    InvalidAsset,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// Try to compute when chain is idle
    fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
      Self::clean_proposal_queue_with_max_weight(remaining_weight)
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    /// Quorum member submit proposal
    #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_proposal())]
    pub fn submit_proposal(
      origin: OriginFor<T>,
      proposal: ProposalType<T::AccountId, T::BlockNumber, Vec<u8>, Vec<T::AccountId>>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member_and_ready(&sender), Error::<T>::AccessDenied);

      // 3. Add the proposal in queue
      let current_block = T::Security::get_current_block_count();
      let proposal_id = T::Security::get_unique_id(sender);

      // Transform the proposal type to use bounded vector
      let proposal: ProposalType<
        T::AccountId,
        T::BlockNumber,
        BoundedVec<u8, <T as pallet::Config>::StringLimit>,
        BoundedVec<T::AccountId, <T as pallet::Config>::VotesLimit>,
      > = match proposal {
        ProposalType::Mint(mint) => ProposalType::Mint(Mint {
          account_id: mint.account_id,
          currency_id: mint.currency_id,
          mint_amount: mint.mint_amount,
          gas_amount: mint.gas_amount,
          transaction_id: mint
            .transaction_id
            .try_into()
            .map_err(|_| Error::<T>::BadTransactionId)?,
          compliance_level: mint.compliance_level,
        }),
        ProposalType::Withdrawal(withdrawal) => ProposalType::Withdrawal(Withdrawal {
          account_id: withdrawal.account_id,
          asset_id: withdrawal.asset_id,
          amount: withdrawal.amount,
          external_address: withdrawal
            .external_address
            .try_into()
            .map_err(|_| Error::<T>::BadExternalAddress)?,
          block_number: withdrawal.block_number,
        }),
        ProposalType::UpdateConfiguration(members, threshold) => ProposalType::UpdateConfiguration(
          members
            .try_into()
            .map_err(|_| Error::<T>::MembersOverflow)?,
          threshold,
        ),
      };

      Proposals::<T>::try_append((proposal_id, current_block, proposal))
        .map_err(|_| Error::<T>::ProposalsCapExceeded)?;

      // 4. Emit event on chain
      Self::deposit_event(Event::<T>::ProposalSubmitted { proposal_id });

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Quorum member acknowledge to a proposal
    #[pallet::weight(<T as pallet::Config>::WeightInfo::acknowledge_proposal())]
    pub fn acknowledge_proposal(
      origin: OriginFor<T>,
      proposal: Hash,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member_and_ready(&sender), Error::<T>::AccessDenied);

      // 3. Register vote
      Self::vote_for(sender, proposal)?;

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Quorum member acknowledge a burned item and started the process.
    #[pallet::weight(<T as pallet::Config>::WeightInfo::acknowledge_burned())]
    pub fn acknowledge_burned(origin: OriginFor<T>, proposal: Hash) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member_and_ready(&sender), Error::<T>::AccessDenied);

      // 3. Remove from the queue
      BurnedQueue::<T>::mutate(|burned_queue| {
        burned_queue.retain(|(proposal_id, _)| *proposal_id != proposal);
      });

      // 4. Emit event on chain
      Self::deposit_event(Event::<T>::BurnedAcknowledged {
        proposal_id: proposal,
      });

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Quorum member reject a proposal
    #[pallet::weight(<T as pallet::Config>::WeightInfo::reject_proposal())]
    pub fn reject_proposal(origin: OriginFor<T>, proposal: Hash) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member_and_ready(&sender), Error::<T>::AccessDenied);

      // 3. Register vote
      Self::vote_against(sender, proposal)?;

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }

    /// Evaluate the state of a proposal given the current vote threshold
    #[pallet::weight(<T as pallet::Config>::WeightInfo::eval_proposal_state())]
    pub fn eval_proposal_state(origin: OriginFor<T>, proposal: Hash) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      ensure_signed(origin)?;

      // 2. Resolve proposal
      Self::try_resolve_proposal(proposal)?;

      // Transactor will pay related fees
      Ok(Pays::Yes.into())
    }

    /// Quorum member submit his own public keys
    #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_public_keys(public_keys.len() as u32))]
    pub fn submit_public_keys(
      origin: OriginFor<T>,
      public_keys: Vec<(AssetId, Vec<u8>)>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member(&sender), Error::<T>::AccessDenied);

      // 3. Delete all existing public keys of this member
      Self::delete_public_keys_for_account(&sender);

      // 4. Register new public keys
      for (asset_id, public_key) in public_keys {
        Self::add_public_keys_for_asset(&sender, asset_id, public_key)?;
      }

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    fn clean_proposal_queue_with_max_weight(max_weight: Weight) -> Weight {
      let all_proposals = Proposals::<T>::get();
      let mut weight_used = <T as frame_system::Config>::DbWeight::get().reads(1);

      if all_proposals.len() == 0 {
        return weight_used;
      }

      // The amount of remaining weight under which we stop processing messages
      let threshold_weight = 100_000;

      // we create a shuffle of index, to prevent queue blocking
      let mut shuffled = Self::create_shuffle(all_proposals.len());
      let current_block = T::Security::get_current_block_count();
      let proposal_lifetime = T::ProposalLifetime::get();
      let mut shuffle_index = 0;
      let mut weight_available = 0;

      while shuffle_index < shuffled.len()
        && max_weight.saturating_sub(weight_used) >= threshold_weight
      {
        let index = shuffled[shuffle_index];
        let proposal_id = all_proposals[index].0;
        let proposal_expiration = all_proposals[index].1 + proposal_lifetime;

        if weight_available != max_weight {
          // The speed to which the available weight approaches the maximum weight. A lower number
          // results in a faster progression. A value of 1 makes the entire weight available initially.
          let weight_restrict_decay = 2;
          // Get incrementally closer to freeing up max_weight for first round.
          // For the second round we unlock all weight. If we come close enough
          // on the first round to unlocking everything, then we do so.
          if shuffle_index < all_proposals.len() {
            weight_available += (max_weight - weight_available) / (weight_restrict_decay + 1);
            if weight_available + threshold_weight > max_weight {
              weight_available = max_weight;
            }
          } else {
            weight_available = max_weight;
          }
        }

        let weight_processed = if current_block >= proposal_expiration {
          // Delete proposal (1 write)
          if Self::delete_proposal(proposal_id).is_err() {
            log!(error, "Can't delete proposal {}", proposal_id);
          };

          // Delete all votes (1 write)
          Votes::<T>::remove(&proposal_id);

          <T as frame_system::Config>::DbWeight::get().reads_writes(0, 2)
        } else {
          0
        };

        weight_used += weight_processed;

        // If there are more and we're making progress, we process them after we've given the
        // other channels a look in. If we've still not unlocked all weight, then we set them
        // up for processing a second time anyway.
        if current_block >= proposal_expiration
          && (weight_processed > 0 || weight_available != max_weight)
        {
          if shuffle_index + 1 == shuffled.len() {
            // Only this queue left. Just run around this loop once more.
            continue;
          }
          shuffled.push(index);
        }
        shuffle_index += 1;
      }

      weight_used
    }

    // Delete all member public keys
    fn delete_public_keys_for_account(who: &T::AccountId) {
      for asset_id in PublicKeys::<T>::iter_keys() {
        PublicKeys::<T>::mutate(asset_id, |public_keys| {
          public_keys.retain(|(account_id, _)| *account_id != *who);
        });
      }
    }

    // Add member public key for a specific asset id
    fn add_public_keys_for_asset(
      who: &T::AccountId,
      asset_id: AssetId,
      public_key: Vec<u8>,
    ) -> Result<(), DispatchError> {
      let final_public_key = public_key
        .try_into()
        .map_err(|_| Error::<T>::BadPublicKey)?;
      PublicKeys::<T>::try_mutate(asset_id, |public_keys| {
        // Prevent duplicate for the same member / asset id pubkey
        public_keys.retain(|(account_id, _)| *account_id != *who);
        // Add new public key
        public_keys
          .try_push((who.clone(), final_public_key))
          .map_err(|_| Error::<T>::PublicKeysOverflow)?;
        Ok(())
      })
    }

    // Create a shuffled vector the size of `len` with random keys
    pub(crate) fn create_shuffle(len: usize) -> Vec<usize> {
      // Create a shuffled order for use to iterate through.
      // Not a great random seed, but good enough for our purposes.
      let seed = frame_system::Pallet::<T>::parent_hash();
      let seed = <[u8; 32]>::decode(&mut sp_runtime::traits::TrailingZeroInput::new(
        seed.as_ref(),
      ))
      .expect("input is padded with zeroes; qed");
      let mut rng = ChaChaRng::from_seed(seed);
      let mut shuffled = (0..len).collect::<Vec<_>>();
      for i in 0..len {
        let j = (rng.next_u32() as usize) % len;
        shuffled.swap(i, j);
      }
      shuffled
    }

    // Make sure the account id is part of the quorum set list
    fn is_member(who: &T::AccountId) -> bool {
      Self::members(who).unwrap_or(false)
    }

    // Make sure the account id is part of the quorum set list and have public key set
    fn is_member_and_ready(who: &T::AccountId) -> bool {
      let at_least_one_public_key = PublicKeys::<T>::iter_values()
        .find(|assets| assets.iter().any(|(account_id, _)| account_id == who))
        .is_some();

      Self::members(who).unwrap_or(false) && at_least_one_public_key
    }

    // Register a vote for the proposal
    fn vote_for(who: T::AccountId, proposal_id: Hash) -> Result<(), DispatchError> {
      Self::commit_vote(who, proposal_id, true)?;
      Self::try_resolve_proposal(proposal_id)?;
      Ok(())
    }

    // Register a vote against the proposal
    fn vote_against(who: T::AccountId, proposal_id: Hash) -> Result<(), DispatchError> {
      Self::commit_vote(who, proposal_id, false)?;
      Self::try_resolve_proposal(proposal_id)?;
      Ok(())
    }

    // Record the vote in the storage
    fn commit_vote(who: T::AccountId, proposal_id: Hash, in_favour: bool) -> DispatchResult {
      let current_block = T::Security::get_current_block_count();
      let proposal_block = Self::proposals()
        .into_iter()
        .find(|(id, _, _)| *id == proposal_id)
        .ok_or(Error::<T>::ProposalDoesNotExist)?
        .1;

      ensure!(
        current_block >= proposal_block,
        Error::<T>::ProposalBlockIsInFuture
      );

      let mut votes = Votes::<T>::get(proposal_id).unwrap_or_else(|| {
        let mut v =
          ProposalVotes::<T::BlockNumber, BoundedVec<T::AccountId, T::VotesLimit>>::default();
        v.expiry = proposal_block + T::ProposalLifetime::get();
        v
      });

      ensure!(
        votes.status == ProposalStatus::Initiated,
        Error::<T>::ProposalAlreadyComplete
      );
      ensure!(votes.expiry >= current_block, Error::<T>::ProposalExpired);
      ensure!(
        !votes.votes_for.contains(&who) && !votes.votes_against.contains(&who),
        Error::<T>::MemberAlreadyVoted
      );

      if in_favour {
        votes
          .votes_for
          .try_push(who.clone())
          .map_err(|_| Error::<T>::VotesForOverflow)?;
        Self::deposit_event(Event::<T>::VoteFor {
          account_id: who,
          proposal_id,
        });
      } else {
        votes
          .votes_against
          .try_push(who.clone())
          .map_err(|_| Error::<T>::VotesAgainstOverflow)?;

        Self::deposit_event(Event::<T>::VoteAgainst {
          account_id: who,
          proposal_id,
        });
      }

      Votes::<T>::insert(proposal_id, votes);

      Ok(())
    }

    // Process proposal
    //
    // A proposal with enough votes will be either executed or cancelled, and the status
    // will be updated accordingly.
    fn try_resolve_proposal(proposal_id: Hash) -> Result<(), Error<T>> {
      Votes::<T>::mutate_exists(proposal_id, |proposal_votes| match proposal_votes {
        Some(votes) => {
          let block_number = T::Security::get_current_block_count();
          ensure!(
            votes.status == ProposalStatus::Initiated,
            Error::<T>::ProposalAlreadyComplete
          );
          ensure!(votes.expiry >= block_number, Error::<T>::ProposalExpired);

          let threshold = Self::threshold();
          let total_members = Members::<T>::count() as u16;
          if votes.votes_for.len() >= threshold as usize {
            Self::deposit_event(Event::<T>::ProposalApproved { proposal_id });
            Self::process_proposal(proposal_id)?;
            Self::delete_proposal(proposal_id)?;
            *proposal_votes = None;
          } else if total_members >= threshold
            && votes.votes_against.len() as u16 + threshold > total_members
          {
            // FIXME: Maybe add some slashing for the proposer?
            Self::deposit_event(Event::<T>::ProposalRejected { proposal_id });
            Self::delete_proposal(proposal_id)?;
            *proposal_votes = None;
          }
          Ok(())
        }
        None => Err(Error::<T>::ProposalDoesNotExist),
      })
    }

    // Process the original proposal call
    fn process_proposal(proposal_id: Hash) -> Result<(), Error<T>> {
      match Self::get_proposal(proposal_id)? {
        // mint token
        ProposalType::Mint(mint) => Self::process_mint(proposal_id, &mint)?,
        // withdraw (burn)
        ProposalType::Withdrawal(withdrawal) => Self::process_withdrawal(proposal_id, &withdrawal)?,
        // update quorum configuration (threshold & member set)
        ProposalType::UpdateConfiguration(members, threshold) => {
          Self::process_update_configuration(&members, threshold)
        }
      };
      Self::deposit_event(Event::<T>::ProposalProcessed { proposal_id });
      Ok(())
    }

    // Process the original proposal call
    pub fn get_proposal(
      proposal_id: Hash,
    ) -> Result<
      ProposalType<
        T::AccountId,
        T::BlockNumber,
        BoundedVec<u8, <T as pallet::Config>::StringLimit>,
        BoundedVec<T::AccountId, <T as pallet::Config>::VotesLimit>,
      >,
      Error<T>,
    > {
      let proposals = Proposals::<T>::get();
      let proposal = proposals
        .iter()
        .find(|(hash, _, _)| *hash == proposal_id)
        .ok_or(Error::<T>::ProposalDoesNotExist)?;
      // FIXME: would be great to add a lifetime to the pallelt
      Ok(proposal.2.clone())
    }

    // Process withdrawal
    fn process_withdrawal(
      proposal_id: Hash,
      item: &Withdrawal<
        T::AccountId,
        T::BlockNumber,
        BoundedVec<u8, <T as pallet::Config>::StringLimit>,
      >,
    ) -> Result<(), Error<T>> {
      // 1. Make sure the currency_id exist and is enabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(item.asset_id),
        Error::<T>::AssetDisabled
      );

      // 2. Remove the token from the account
      T::CurrencyTidefi::burn_from(item.asset_id, &item.account_id, item.amount)
        .map_err(|_| Error::<T>::BurnFailed)?;

      // 3. Add to burned queue, the quorum can poll and initiate the chain deposit
      BurnedQueue::<T>::try_mutate(|burned_queue| {
        burned_queue
          .try_push((proposal_id, item.clone()))
          .map_err(|_| Error::<T>::BurnedQueueOverflow)
      })?;

      // 4. Emit the event on chain
      Self::deposit_event(Event::<T>::BurnedInitialized {
        proposal_id,
        account_id: item.account_id.clone(),
        currency_id: item.asset_id,
        amount: item.amount,
      });

      Ok(())
    }

    // Process mint
    fn process_mint(
      proposal_id: Hash,
      item: &Mint<T::AccountId, BoundedVec<u8, <T as pallet::Config>::StringLimit>>,
    ) -> Result<(), Error<T>> {
      // 1. Make sure the currency_id exist and is enabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(item.currency_id),
        Error::<T>::AssetDisabled
      );

      // 2. Add `Amber` and `Red` to watch list
      if item.compliance_level == ComplianceLevel::Amber
        || item.compliance_level == ComplianceLevel::Red
      {
        Self::add_account_watch_list(
          &item.account_id,
          item.currency_id,
          item.mint_amount,
          item.compliance_level.clone(),
          item.transaction_id.clone().to_vec(),
          WatchListAction::Mint,
        )?;
      }

      // 3. Mint `Green` and `Amber`
      if item.compliance_level == ComplianceLevel::Green
        || item.compliance_level == ComplianceLevel::Amber
      {
        T::CurrencyTidefi::mint_into(item.currency_id, &item.account_id, item.mint_amount)
          .map_err(|_| Error::<T>::MintFailed)?;

        // 3 a. If Quorum provide `gas_amount` try to process refunds based on sunrise allocation
        if let Some(gas_amount) = item.gas_amount {
          // gas for USDT by example, are paid in ETH
          // we extract the base chain for the asset
          // and if needed extract the currency id

          // quorum would have sent us the amount in ETH
          // but the mint would have been for `USDT`
          let asset_from: Asset = item
            .currency_id
            .try_into()
            .map_err(|_| Error::<T>::InvalidAsset)?;

          let real_currency_id = match asset_from.base_chain() {
            Some(base_chain) => base_chain.currency_id(),
            None => item.currency_id,
          };
          if let Err(refund_error) =
            T::Sunrise::try_refund_gas_for_deposit(&item.account_id, real_currency_id, gas_amount)
          {
            log!(error, "Unable to process gas refund {:?}", refund_error);
          }
        }
        // item.gas_amount

        Self::deposit_event(Event::<T>::Minted {
          proposal_id,
          account_id: item.account_id.clone(),
          currency_id: item.currency_id,
          amount: item.mint_amount,
          transaction_id: item.transaction_id.clone().to_vec(),
          compliance_level: item.compliance_level.clone(),
        });
      }

      Ok(())
    }

    // Process configuration update
    fn process_update_configuration(members: &Vec<T::AccountId>, threshold: u16) {
      // 1. Remove all members existing
      let _ = Members::<T>::clear(u32::MAX, None);

      // 2. Remove all public keys
      //
      // FIXME: We need to validate if we want to have quorum to resubmit keys?
      let _ = PublicKeys::<T>::clear(u32::MAX, None);

      // 3. Add new set
      for account in members {
        Members::<T>::insert(account, true);
      }

      // 4. Update threshold
      Threshold::<T>::put(threshold);

      // 5. Emit event
      Self::deposit_event(Event::<T>::ConfigurationUpdated {
        threshold,
        members: members.clone(),
      });
    }

    // Delete specific proposal
    fn delete_proposal(proposal_id: Hash) -> Result<(), Error<T>> {
      Proposals::<T>::mutate(|proposals| {
        proposals.retain(|(found_proposal_id, _, _)| *found_proposal_id != proposal_id);
        Ok(())
      })
    }

    // Add new account to watch list
    fn add_account_watch_list(
      account_id: &T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      compliance_level: ComplianceLevel,
      transaction_id: Vec<u8>,
      watch_action: WatchListAction,
    ) -> Result<(), Error<T>> {
      let block_number = T::Security::get_current_block_count();
      let transaction_id: BoundedVec<u8, <T as pallet::Config>::StringLimit> = transaction_id
        .try_into()
        .map_err(|_| Error::<T>::BadTransactionId)?;

      let watch_list = WatchList {
        amount,
        block_number,
        compliance_level: compliance_level.clone(),
        currency_id,
        watch_action: watch_action.clone(),
        transaction_id: transaction_id.clone(),
      };

      AccountWatchList::<T>::try_mutate_exists(account_id, |account_watch_list| {
        match account_watch_list {
          Some(current_watch_list) => current_watch_list
            .try_push(watch_list.clone())
            .map_err(|_| Error::<T>::WatchlistOverflow),
          None => {
            *account_watch_list = Some(
              vec![watch_list]
                .try_into()
                .expect("Watch list should be created"),
            );
            Ok(())
          }
        }
      })?;

      Self::deposit_event(Event::<T>::WatchTransactionAdded {
        account_id: account_id.clone(),
        currency_id,
        amount,
        compliance_level,
        watch_action,
        transaction_id: transaction_id.to_vec(),
      });

      Ok(())
    }
  }

  // quorum extension exposed in other pallets
  impl<T: Config> QuorumExt<T::AccountId, T::BlockNumber> for Pallet<T> {
    /// Get quorum status
    fn is_quorum_enabled() -> bool {
      T::Security::is_chain_running() && Self::status()
    }

    /// Add new withdrawal in queue
    fn add_new_withdrawal_in_queue(
      account_id: T::AccountId,
      asset_id: CurrencyId,
      amount: Balance,
      external_address: Vec<u8>,
    ) -> Result<(), DispatchError> {
      let unique_id = T::Security::get_unique_id(account_id.clone());
      let block_number = T::Security::get_current_block_count();

      let external_address: BoundedVec<u8, <T as pallet::Config>::StringLimit> = external_address
        .try_into()
        .map_err(|_| Error::<T>::BadExternalAddress)?;

      Proposals::<T>::try_append((
        unique_id,
        block_number,
        ProposalType::Withdrawal(Withdrawal {
          account_id,
          amount,
          asset_id,
          external_address,
          block_number,
        }),
      ))
      .map_err(|_| Error::<T>::ProposalsCapExceeded)?;

      Ok(())
    }
  }
}
