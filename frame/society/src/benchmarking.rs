// This file is part of Substrate.

// Copyright (C) 2021-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Society pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks_instance_pallet, whitelisted_caller};
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;

use crate::Pallet as Society;

// Set up Society
fn setup_society<T: Config<I>, I: 'static>(founder: T::AccountId) -> Result<(), &'static str> {
	let candidate_deposit = T::Currency::minimum_balance();
	let origin = T::FounderSetOrigin::successful_origin();
	let max_members = 5u32.into();
	let max_intake = 3u32.into();
	let max_strikes = 3u32.into();
	Society::<T, I>::found_society(
		origin,
		founder,
		max_members,
		max_intake,
		max_strikes,
		candidate_deposit,
		b"benchmarking-society".to_vec()
	)?;
	Ok(())
}

benchmarks_instance_pallet! {

	bid {
		let founder: T::AccountId = account("founder", 0, 0);
		setup_society::<T, I>(founder.clone())?;

		let caller: T::AccountId = whitelisted_caller();
		let deposit: BalanceOf<T, I> = T::Currency::minimum_balance();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());
	}: bid(RawOrigin::Signed(caller.clone()), 10u32.into())
	verify {
		let first_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: caller.clone(),
			kind: BidKind::Deposit(deposit),
			value: 10u32.into(),
		};
		assert_eq!(Bids::<T, I>::get(), vec![first_bid]);
	}

	unbid {
		let founder: T::AccountId = account("founder", 0, 0);
		setup_society::<T, I>(founder.clone())?;

		let caller: T::AccountId = whitelisted_caller();
		let deposit: BalanceOf<T, I> = T::Currency::minimum_balance();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());

		let mut bids = Bids::<T, I>::get();
		Society::<T, I>::insert_bid(&mut bids, &caller, 10u32.into(), BidKind::Deposit(deposit));
		Bids::<T, I>::put(bids);
	}: unbid(RawOrigin::Signed(caller.clone()))
	verify {
		assert_eq!(Bids::<T, I>::get(), vec![]);
	}

	found_society {
		let founder: T::AccountId = whitelisted_caller();
		let can_found = T::FounderSetOrigin::successful_origin();
		let candidate_deposit: BalanceOf<T, I> = T::Currency::minimum_balance();
	}: found_society<T::Origin>(can_found, founder.clone(), 5, 3, 3, candidate_deposit, b"benchmarking-society".to_vec())
	verify {
		assert_eq!(<Founder<T, I>>::get(), Some(founder.clone()));
	}

	dissolve {
		let founder: T::AccountId = whitelisted_caller();
		setup_society::<T, I>(founder.clone())?;
	}: dissolve(RawOrigin::Signed(founder))
	verify {
		assert_eq!(<Founder<T, I>>::get(), None);
	}

	impl_benchmark_test_suite!(
		Society,
		crate::tests_composite::ExtBuilder::default().build(),
		crate::tests_composite::Test,
	)
}
