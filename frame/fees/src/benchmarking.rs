//! Benchmarking setup for pallet-fees
use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{fungibles::Mutate, Hooks};
use frame_system::{self, RawOrigin};
use tidefi_primitives::{pallet::FeesExt, CurrencyId};

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
  claim_sunrise_rewards {
    let caller: T::AccountId = whitelisted_caller();
    // init first era
    Pallet::<T>::start_era();
    Pallet::<T>::on_finalize(T::BlockNumber::from(1_u32));

    T::CurrencyTidefi::mint_into(CurrencyId::Tide, &Pallet::<T>::account_id(), 1_000_000_000_000_000_000).unwrap();
    SunriseRewards::<T>::insert(&caller, 1, 1_000_000_000_000);

    frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
    frame_system::Pallet::<T>::set_block_number(
      frame_system::Pallet::<T>::block_number() + T::BlockNumber::from(100_000_u32),
    );
    frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
    frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
    pallet_security::CurrentBlockCount::<T>::put(frame_system::Pallet::<T>::block_number());

 }: _(RawOrigin::Signed(caller), 1)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
