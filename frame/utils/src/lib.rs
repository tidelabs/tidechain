#[macro_export]
macro_rules! construct_mock_runtime {
	{
		{
			$( $runtime:tt )*
		},
      {
         $( $parameters:tt )*
      }
	} => {
      use codec::{Decode, Encode, MaxEncodedLen};
      use sp_runtime::{
         testing::Header,
         traits::{BlakeTwo256, IdentityLookup, Zero},
         DispatchError, DispatchResult, FixedPointNumber, FixedU128, Percent, Permill, RuntimeDebug,
      };
      use std::marker::PhantomData;
      #[cfg(feature = "std")]
      use serde::{Deserialize, Serialize};
      use frame_support::{
         parameter_types,
         traits::{
           fungible::{
             Inspect as FungibleInspect, InspectHold as FungibleInspectHold, Mutate as FungibleMutate,
             MutateHold as FungibleMutateHold, Transfer as FungibleTransfer,
           },
           fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
           tokens::{DepositConsequence, WithdrawConsequence},
           ConstU32, GenesisBuild,
         },
      };

      pub const TDFY: Balance = 1_000_000_000_000;
      type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
      type Block = frame_system::mocking::MockBlock<Test>;
      type Balance = u128;

      #[derive(
        Encode,
        Decode,
        Default,
        Eq,
        PartialEq,
        Copy,
        Clone,
        RuntimeDebug,
        PartialOrd,
        Ord,
        MaxEncodedLen,
        scale_info::TypeInfo,
      )]
      #[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
      pub struct AccountId(pub u64);

      impl sp_std::fmt::Display for AccountId {
        fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
          write!(f, "{}", self.0)
        }
      }

      impl From<u64> for AccountId {
        fn from(account_id: u64) -> Self {
          Self(account_id)
        }
      }
      impl From<u32> for AccountId {
        fn from(account_id: u32) -> Self {
          Self(account_id as u64)
        }
      }
      // random adresss bytes (0 * 1) + 31
      impl From<[u8; 32]> for AccountId {
         fn from(account_id: [u8; 32]) -> Self {
           Self(account_id[0].saturating_mul(account_id[1]).saturating_add(account_id[31]) as u64)
         }
       }

      // Configure a mock runtime to test the pallet.
      frame_support::construct_runtime!(
        pub enum Test where
          Block = Block,
          NodeBlock = Block,
          UncheckedExtrinsic = UncheckedExtrinsic,
        {
          System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
          Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
          Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
          $( $runtime )*
        }
      );

      parameter_types! {
         pub const BlockHashCount: u64 = 250;
         pub const SS58Prefix: u8 = 42;
         pub const MinimumPeriod: u64 = 5;
         pub const ExistentialDeposit: Balance = TDFY;
         pub const MaxLocks: u32 = 50;
         pub const MaxReserves: u32 = 50;
         $( $parameters )*
       }

       impl system::Config for Test {
         type BaseCallFilter = frame_support::traits::Everything;
         type BlockWeights = ();
         type BlockLength = ();
         type DbWeight = ();
         type RuntimeOrigin = RuntimeOrigin;
         type RuntimeCall = RuntimeCall;
         type Index = u64;
         type BlockNumber = u64;
         type Hash = tidefi_primitives::Hash;
         type Hashing = BlakeTwo256;
         type AccountId = AccountId;
         type Lookup = IdentityLookup<Self::AccountId>;
         type Header = Header;
         type RuntimeEvent = RuntimeEvent;
         type BlockHashCount = BlockHashCount;
         type Version = ();
         type PalletInfo = PalletInfo;
         type AccountData = pallet_balances::AccountData<Balance>;
         type OnNewAccount = ();
         type OnKilledAccount = ();
         type SystemWeightInfo = ();
         type SS58Prefix = SS58Prefix;
         type OnSetCode = ();
         type MaxConsumers = ConstU32<16>;
       }

       impl pallet_timestamp::Config for Test {
         type Moment = u64;
         type OnTimestampSet = ();
         type MinimumPeriod = MinimumPeriod;
         type WeightInfo = ();
       }

       impl pallet_balances::Config for Test {
         type Balance = Balance;
         type DustRemoval = ();
         type RuntimeEvent = RuntimeEvent;
         type ExistentialDeposit = ExistentialDeposit;
         type AccountStore = frame_system::Pallet<Test>;
         type MaxLocks = MaxLocks;
         type MaxReserves = MaxReserves;
         type ReserveIdentifier = [u8; 8];
         type WeightInfo = ();
       }


      pub struct Adapter<AccountId> {
         phantom: PhantomData<AccountId>,
      }

      impl Inspect<AccountId> for Adapter<AccountId> {
         type AssetId = CurrencyId;
         type Balance = Balance;

         fn total_issuance(asset: Self::AssetId) -> Self::Balance {
            match asset {
               CurrencyId::Tdfy => Balances::total_issuance(),
               CurrencyId::Wrapped(asset_id) => Assets::total_issuance(asset_id),
            }
         }

         fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
            match asset {
               CurrencyId::Tdfy => Balances::balance(who),
               CurrencyId::Wrapped(asset_id) => Assets::balance(asset_id, who),
            }
         }

         fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
            match asset {
               CurrencyId::Tdfy => Balances::minimum_balance(),
               CurrencyId::Wrapped(asset_id) => Assets::minimum_balance(asset_id),
            }
         }

         fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
            match asset {
               CurrencyId::Tdfy => Balances::reducible_balance(who, keep_alive),
               CurrencyId::Wrapped(asset_id) => Assets::reducible_balance(asset_id, who, keep_alive),
            }
         }

         fn can_deposit(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Self::Balance,
            mint: bool,
         ) -> DepositConsequence {
            match asset {
               CurrencyId::Tdfy => Balances::can_deposit(who, amount, mint),
               CurrencyId::Wrapped(asset_id) => Assets::can_deposit(asset_id, who, amount, mint),
            }
         }

         fn can_withdraw(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Self::Balance,
         ) -> WithdrawConsequence<Self::Balance> {
            match asset {
               CurrencyId::Tdfy => Balances::can_withdraw(who, amount),
               CurrencyId::Wrapped(asset_id) => Assets::can_withdraw(asset_id, who, amount),
            }
         }

         fn asset_exists(asset: Self::AssetId) -> bool {
            match asset {
              CurrencyId::Tdfy => true,
              CurrencyId::Wrapped(asset_id) => Assets::asset_exists(asset_id),
            }
          }
      }

      impl Mutate<AccountId> for Adapter<AccountId> {
         fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
            match asset {
               CurrencyId::Tdfy => Balances::mint_into(who, amount),
               CurrencyId::Wrapped(asset_id) => Assets::mint_into(asset_id, who, amount),
            }
         }

         fn burn_from(
            asset: Self::AssetId,
            who: &AccountId,
            amount: Balance,
         ) -> Result<Balance, DispatchError> {
            match asset {
               CurrencyId::Tdfy => Balances::burn_from(who, amount),
               CurrencyId::Wrapped(asset_id) => Assets::burn_from(asset_id, who, amount),
            }
         }
      }

      impl Transfer<AccountId> for Adapter<AccountId>
      where
      Assets: Transfer<AccountId>,
      {
         fn transfer(
            asset: Self::AssetId,
            source: &AccountId,
            dest: &AccountId,
            amount: Self::Balance,
            keep_alive: bool,
         ) -> Result<Balance, DispatchError> {
            match asset {
               CurrencyId::Tdfy => {
               <Balances as FungibleTransfer<AccountId>>::transfer(source, dest, amount, keep_alive)
               }
               CurrencyId::Wrapped(asset_id) => {
               <Assets as Transfer<AccountId>>::transfer(asset_id, source, dest, amount, keep_alive)
               }
            }
         }
      }

      impl InspectHold<AccountId> for Adapter<AccountId> {
         fn balance_on_hold(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
            match asset {
               CurrencyId::Tdfy => Balances::balance_on_hold(who),
               CurrencyId::Wrapped(asset_id) => Assets::balance_on_hold(asset_id, who),
            }
         }
         fn can_hold(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> bool {
            match asset {
               CurrencyId::Tdfy => Balances::can_hold(who, amount),
               CurrencyId::Wrapped(asset_id) => Assets::can_hold(asset_id, who, amount),
            }
         }
      }

      impl MutateHold<AccountId> for Adapter<AccountId> {
         fn hold(asset: CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
            match asset {
               CurrencyId::Tdfy => Balances::hold(who, amount),
               CurrencyId::Wrapped(asset_id) => Assets::hold(asset_id, who, amount),
            }
         }

         fn release(
            asset: CurrencyId,
            who: &AccountId,
            amount: Balance,
            best_effort: bool,
         ) -> Result<Balance, DispatchError> {
            if amount.is_zero() {
               return Ok(amount);
            }
            match asset {
               CurrencyId::Tdfy => Balances::release(who, amount, best_effort),
               CurrencyId::Wrapped(asset_id) => Assets::release(asset_id, who, amount, best_effort),
            }
         }
         fn transfer_held(
            asset: CurrencyId,
            source: &AccountId,
            dest: &AccountId,
            amount: Balance,
            best_effort: bool,
            on_hold: bool,
         ) -> Result<Balance, DispatchError> {
            match asset {
               CurrencyId::Tdfy => Balances::transfer_held(source, dest, amount, best_effort, on_hold),
               CurrencyId::Wrapped(asset_id) => {
               Assets::transfer_held(asset_id, source, dest, amount, best_effort, on_hold)
               }
            }
         }
      }
	}
}
