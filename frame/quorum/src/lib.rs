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
    inherent::Vec,
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
  use sp_std::vec;
  use tidefi_primitives::{
    pallet::{AssetRegistryExt, QuorumExt, SecurityExt},
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

    /// Proposals lifetime
    #[pallet::constant]
    type ProposalLifetime: Get<Self::BlockNumber>;

    /// Weights
    type WeightInfo: WeightInfo;

    /// Security traits
    type Security: SecurityExt<Self::AccountId, Self::BlockNumber>;

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
  pub type PublicKeys<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    AssetId,
    Vec<u8>,
    ValueQuery,
  >;

  /// Set of active transaction to watch
  #[pallet::storage]
  #[pallet::getter(fn account_watch_list)]
  pub type AccountWatchList<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<WatchList<T::BlockNumber>>>;

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
        ProposalType<T::AccountId, T::BlockNumber>,
      ),
      T::ProposalsCap,
    >,
    ValueQuery,
  >;

  /// Set of Votes for each proposal
  #[pallet::storage]
  #[pallet::getter(fn proposal_votes)]
  pub type Votes<T: Config> =
    CountedStorageMap<_, Blake2_128Concat, Hash, ProposalVotes<T::AccountId, T::BlockNumber>>;

  /// Set of active quorum members
  #[pallet::storage]
  #[pallet::getter(fn members)]
  pub type Members<T: Config> = CountedStorageMap<_, Blake2_128Concat, T::AccountId, bool>;

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
      for account_id in self.members.clone() {
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
    /// Quorum burned token to the account
    Burned {
      proposal_id: Hash,
      account_id: T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
    },

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
    /// Proposal has either failed or succeeded
    ProposalAlreadyComplete,
    /// Lifetime of proposal has been exceeded
    ProposalExpired,
    /// Member already voted for this proposal
    MemberAlreadyVoted,
    /// Mint failed
    MintFailed,
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
      proposal: ProposalType<T::AccountId, T::BlockNumber>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member_and_ready(&sender), Error::<T>::AccessDenied);

      // 3. Add the proposal in queue
      let current_block = T::Security::get_current_block_count();
      let proposal_id = T::Security::get_unique_id(sender.clone());
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
    #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_public_keys())]
    pub fn submit_public_keys(
      origin: OriginFor<T>,
      public_keys: Vec<(AssetId, Vec<u8>)>,
    ) -> DispatchResultWithPostInfo {
      // 1. Make sure the request is signed by `account_id`
      let sender = ensure_signed(origin)?;

      // 2. Make sure this is a quorum member
      ensure!(Self::is_member(&sender), Error::<T>::AccessDenied);

      // 3. Delete all existing public keys of this member
      PublicKeys::<T>::remove_prefix(&sender, None);

      // 4. Register new public keys
      for (asset_id, public_key) in public_keys {
        PublicKeys::<T>::insert(&sender, asset_id, public_key);
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
          if let Err(_) = Self::delete_proposal(proposal_id) {
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
        let a = shuffled[i];
        shuffled[i] = shuffled[j];
        shuffled[j] = a;
      }
      shuffled
    }

    // Make sure the account id is part of the quorum set list
    fn is_member(who: &T::AccountId) -> bool {
      Self::members(who).unwrap_or(false)
    }

    // Make sure the account id is part of the quorum set list and have public key set
    fn is_member_and_ready(who: &T::AccountId) -> bool {
      Self::members(who).unwrap_or(false) && PublicKeys::<T>::iter_key_prefix(who).count() > 0
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
      let block_number = T::Security::get_current_block_count();
      let mut votes = Votes::<T>::get(proposal_id).unwrap_or_else(|| {
        let mut v = ProposalVotes::default();
        v.expiry = block_number + T::ProposalLifetime::get();
        v
      });

      ensure!(
        votes.status == ProposalStatus::Initiated,
        Error::<T>::ProposalAlreadyComplete
      );
      ensure!(votes.expiry >= block_number, Error::<T>::ProposalExpired);
      ensure!(
        !votes.votes_for.contains(&who) || !votes.votes_against.contains(&who),
        Error::<T>::MemberAlreadyVoted
      );

      if in_favour {
        votes.votes_for.push(who.clone());
        Self::deposit_event(Event::<T>::VoteFor {
          account_id: who,
          proposal_id,
        });
      } else {
        votes.votes_against.push(who.clone());
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
          let status = if votes.votes_for.len() >= threshold as usize {
            votes.status = ProposalStatus::Approved;
            ProposalStatus::Approved
          } else if total_members >= threshold
            && votes.votes_against.len() as u16 + threshold > total_members
          {
            votes.status = ProposalStatus::Rejected;
            ProposalStatus::Rejected
          } else {
            ProposalStatus::Initiated
          };

          *proposal_votes = Some(votes.clone());
          match status {
            ProposalStatus::Approved => {
              Self::deposit_event(Event::<T>::ProposalApproved { proposal_id });
              Self::process_proposal(proposal_id)?;
              Self::delete_proposal(proposal_id)?;
              *proposal_votes = None;
              Ok(())
            }
            ProposalStatus::Rejected => {
              // FIXME: Maybe add some slashing for the proposer?
              Self::deposit_event(Event::<T>::ProposalRejected { proposal_id });
              Self::delete_proposal(proposal_id)?;
              *proposal_votes = None;
              Ok(())
            }
            _ => Ok(()),
          }
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
          Self::process_update_configuration(&members, threshold)?
        }
      };
      Self::deposit_event(Event::<T>::ProposalProcessed { proposal_id });
      Ok(())
    }

    // Process the original proposal call
    pub fn get_proposal(
      proposal_id: Hash,
    ) -> Result<ProposalType<T::AccountId, T::BlockNumber>, Error<T>> {
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
      item: &Withdrawal<T::AccountId, T::BlockNumber>,
    ) -> Result<(), Error<T>> {
      // 1. Make sure the currency_id exist and is enabled
      ensure!(
        T::AssetRegistry::is_currency_enabled(item.asset_id),
        Error::<T>::AssetDisabled
      );

      // 2. Remove the token from the account
      T::CurrencyTidefi::burn_from(item.asset_id, &item.account_id, item.amount)
        .map_err(|_| Error::<T>::BurnFailed)?;

      // 3. Emit the event on chain
      Self::deposit_event(Event::<T>::Burned {
        proposal_id,
        account_id: item.account_id.clone(),
        currency_id: item.asset_id,
        amount: item.amount,
      });

      Ok(())
    }

    // Process mint
    fn process_mint(proposal_id: Hash, item: &Mint<T::AccountId>) -> Result<(), Error<T>> {
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
          item.transaction_id.clone(),
          WatchListAction::Mint,
        );
      }

      // 3. Mint `Green` and `Amber`
      if item.compliance_level == ComplianceLevel::Green
        || item.compliance_level == ComplianceLevel::Amber
      {
        T::CurrencyTidefi::mint_into(item.currency_id, &item.account_id, item.mint_amount)
          .map_err(|_| Error::<T>::MintFailed)?;

        Self::deposit_event(Event::<T>::Minted {
          proposal_id,
          account_id: item.account_id.clone(),
          currency_id: item.currency_id,
          amount: item.mint_amount,
          transaction_id: item.transaction_id.clone(),
          compliance_level: item.compliance_level.clone(),
        });
      }

      Ok(())
    }

    // Process configuration update
    fn process_update_configuration(
      members: &Vec<T::AccountId>,
      threshold: u16,
    ) -> Result<(), Error<T>> {
      // 1. Remove all members existing
      Members::<T>::remove_all();

      // 2. Remove all public keys
      //
      // FIXME: We need to validate if we want to have quorum to resubmit keys?
      PublicKeys::<T>::remove_all(None);

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

      Ok(())
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
    ) {
      let block_number = T::Security::get_current_block_count();
      let watch_list = WatchList {
        amount,
        block_number,
        compliance_level: compliance_level.clone(),
        currency_id,
        watch_action: watch_action.clone(),
        transaction_id: transaction_id.clone(),
      };

      AccountWatchList::<T>::mutate_exists(
        account_id,
        |account_watch_list| match account_watch_list {
          Some(current_watch_list) => current_watch_list.push(watch_list),
          None => AccountWatchList::<T>::insert(account_id, vec![watch_list]),
        },
      );

      Self::deposit_event(Event::<T>::WatchTransactionAdded {
        account_id: account_id.clone(),
        currency_id,
        amount,
        compliance_level,
        watch_action,
        transaction_id,
      });
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
