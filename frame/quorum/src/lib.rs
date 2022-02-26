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

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::{
    inherent::Vec,
    pallet_prelude::*,
    traits::fungibles::{Inspect, Mutate, Transfer},
    PalletId,
  };
  use frame_system::pallet_prelude::*;
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
    BoundedVec<(Hash, ProposalType<T::AccountId, T::BlockNumber>), T::ProposalsCap>,
    ValueQuery,
  >;

  /// Set of Votes for each proposal
  #[pallet::storage]
  #[pallet::getter(fn proposal_votes)]
  pub type Votes<T: Config> =
    StorageMap<_, Blake2_128Concat, Hash, ProposalVotes<T::AccountId, T::BlockNumber>>;

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

    /// Proposal has been processed successfully
    ProposalApproved { proposal_id: Hash },

    /// Proposal has been rejected
    ProposalRejected { proposal_id: Hash },

    /// The quorum configuration has been updated
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
    /// The Quorum is paused. Try again later.
    QuorumPaused,
    /// The access to the Quorum pallet is not allowed for this account ID.
    AccessDenied,
    /// Invalid request ID.
    InvalidRequestId,
    /// There is a conflict in the request.
    Conflict,
    /// Unable to burn token.
    BurnFailed,
    /// Proposals cap exceeded, try again later.
    ProposalsCapExceeded,
    /// No proposal with the ID was found
    ProposalDoesNotExist,
    /// Cannot complete proposal, needs more votes
    ProposalNotComplete,
    /// Proposal has either failed or succeeded
    ProposalAlreadyComplete,
    /// Lifetime of proposal has been exceeded
    ProposalExpired,
    /// Member already voted for this proposal
    MemberAlreadyVoted,
    /// Mint failed
    MintFailed,
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
      ensure!(Self::is_member(&sender), Error::<T>::AccessDenied);

      // 3. Add the proposal in queue
      let proposal_id = T::Security::get_unique_id(sender.clone());
      Proposals::<T>::try_append((proposal_id, proposal))
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
      ensure!(Self::is_member(&sender), Error::<T>::AccessDenied);

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
      ensure!(Self::is_member(&sender), Error::<T>::AccessDenied);

      // 3. Register vote
      Self::vote_against(sender, proposal)?;

      // Don't take tx fees on success
      Ok(Pays::No.into())
    }
  }

  // helper functions (not dispatchable)
  impl<T: Config> Pallet<T> {
    // Make sure the account id is part of the quorum set list
    fn is_member(who: &T::AccountId) -> bool {
      Self::members(who).unwrap_or(false)
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
      let block_number = <frame_system::Pallet<T>>::block_number();
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

    fn try_resolve_proposal(proposal_id: Hash) -> Result<(), Error<T>> {
      Votes::<T>::mutate_exists(proposal_id, |proposal_votes| match proposal_votes {
        Some(votes) => {
          let block_number = <frame_system::Pallet<T>>::block_number();
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

    // Process the original proposal
    fn process_proposal(request_id: Hash) -> Result<(), Error<T>> {
      let proposals = Proposals::<T>::get();
      let (_, proposal) = proposals
        .iter()
        .find(|(hash, _)| *hash == request_id)
        .ok_or(Error::<T>::ProposalDoesNotExist)?;

      match proposal {
        ProposalType::Mint(mint) => Self::process_mint(request_id, mint),
        ProposalType::Withdrawal(withdrawal) => Self::process_withdrawal(request_id, withdrawal),
        ProposalType::UpdateConfiguration(members, threshold) => {
          Self::update_configuration(members, *threshold)
        }
      }
    }

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

    fn update_configuration(members: &Vec<T::AccountId>, threshold: u16) -> Result<(), Error<T>> {
      Members::<T>::remove_all();
      for account in members {
        Members::<T>::insert(account, true);
      }

      Threshold::<T>::put(threshold);

      Self::deposit_event(Event::<T>::ConfigurationUpdated {
        threshold,
        members: members.clone(),
      });
      Ok(())
    }

    fn delete_proposal(proposal_id: Hash) -> Result<(), Error<T>> {
      Proposals::<T>::mutate(|proposals| {
        proposals.retain(|(found_proposal_id, _)| *found_proposal_id != proposal_id);
        Ok(())
      })
    }

    fn _ensure_not_paused() -> Result<(), DispatchError> {
      if Self::is_quorum_enabled() {
        Ok(())
      } else {
        Err(Error::<T>::QuorumPaused.into())
      }
    }

    fn add_account_watch_list(
      account_id: &T::AccountId,
      currency_id: CurrencyId,
      amount: Balance,
      compliance_level: ComplianceLevel,
      transaction_id: Vec<u8>,
      watch_action: WatchListAction,
    ) {
      let block_number = <frame_system::Pallet<T>>::block_number();
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
      Proposals::<T>::try_append((
        unique_id,
        ProposalType::Withdrawal(Withdrawal {
          account_id,
          amount,
          asset_id,
          external_address,
          block_number: <frame_system::Pallet<T>>::block_number(),
        }),
      ))
      .map_err(|_| Error::<T>::ProposalsCapExceeded)?;

      Ok(())
    }
  }
}
