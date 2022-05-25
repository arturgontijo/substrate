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

benchmarks_instance_pallet! {
	bid {
		let caller: T::AccountId = whitelisted_caller();
		let deposit = T::CandidateDeposit::get();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());

	}: bid(RawOrigin::Signed(caller.clone()), 0u32.into())
	verify {
		let first_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: caller.clone(),
			kind: BidKind::Deposit(deposit),
			value: 0u32.into(),
		};
		assert_eq!(Bids::<T, I>::get(), vec![first_bid]);
	}

	unbid {
		let caller: T::AccountId = whitelisted_caller();
		let deposit = T::CandidateDeposit::get();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());

		let first_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: caller.clone(),
			kind: BidKind::Deposit(deposit),
			value: 0u32.into(),
		};
		<Bids<T, I>>::put(vec![first_bid]);

	}: unbid(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(Bids::<T, I>::get(), vec![]);
	}

	vouch {
		let caller: T::AccountId = whitelisted_caller();
		let vouched: T::AccountId = account("vouched", 0, 0);
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());
		<Members<T, I>>::put(vec![caller.clone()]);

	}: vouch(RawOrigin::Signed(caller.clone()), vouched.clone(), 0u32.into(), 0u32.into())
	verify {
		let bids: Vec<Bid<T::AccountId, BalanceOf<T, I>>> = Bids::<T, I>::get();
		let vouched_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: vouched.clone(),
			kind: BidKind::Vouch(caller.clone(), 0u32.into()),
			value: 0u32.into(),
		};
		assert_eq!(bids, vec![vouched_bid]);
	}

	unvouch {
		let caller: T::AccountId = whitelisted_caller();
		let vouched: T::AccountId = account("vouched", 0, 0);
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());

		<Vouching<T, I>>::insert(caller.clone(), VouchingStatus::Vouching);

		let vouched_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: vouched.clone(),
			kind: BidKind::Vouch(caller.clone(), 0u32.into()),
			value: 0u32.into(),
		};
		<Bids<T, I>>::put(vec![vouched_bid]);

	}: unvouch(RawOrigin::Signed(caller.clone()), 0u32.into())
	verify {
		assert_eq!(Bids::<T, I>::get(), vec![]);
	}

	vote {
		let caller: T::AccountId = whitelisted_caller();
		let candidate: T::AccountId = account("candidate", 0, 0);
		let candidate_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(candidate.clone());
		let deposit = T::CandidateDeposit::get();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());
		<Members<T, I>>::put(vec![caller.clone()]);

		let candidate_bid: Bid<T::AccountId, BalanceOf<T, I>> = Bid {
			who: candidate.clone(),
			kind: BidKind::Deposit(deposit),
			value: 0u32.into(),
		};
		<Candidates<T, I>>::put(vec![candidate_bid]);

	}: vote(RawOrigin::Signed(caller.clone()), candidate_lookup, true)
	verify {
		assert_eq!(<Votes<T, I>>::get(candidate.clone(), caller.clone()), Some(Vote::Approve));
	}

	defender_vote {
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());
		<Members<T, I>>::put(vec![caller.clone()]);

	}: defender_vote(RawOrigin::Signed(caller.clone()), false)
	verify {
		assert_eq!(<DefenderVotes<T, I>>::get(caller.clone()), Some(Vote::Reject));
	}

	payout {
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() * 10u32.into());
		<Members<T, I>>::put(vec![caller.clone()]);

		let payout_value = 1u32;
		let payout_when = 0u32;

		T::Currency::make_free_balance_be(&Society::<T, I>::payouts(), BalanceOf::<T, I>::max_value());

		let mut pot = <Pot<T, I>>::get();
		pot = pot.saturating_add(BalanceOf::<T, I>::max_value());
		<Pot<T, I>>::put(&pot);

		Society::<T, I>::bump_payout(&caller, payout_when.into(), payout_value.into());

	}: payout(RawOrigin::Signed(caller.clone()))
	verify {
		assert_eq!(<Payouts<T, I>>::get(caller.clone()), vec![]);
	}

	found {
		let caller: T::AccountId = whitelisted_caller();
		let founder = T::SuspensionJudgementOrigin::successful_origin();
		let can_found = T::FounderSetOrigin::successful_origin();
		Society::<T, I>::unfound(founder)?;
	}: found<T::Origin>(can_found, caller.clone(), 2, b"benchmarking-society".to_vec())
	verify {
		assert_eq!(<Founder<T, I>>::get(), Some(caller.clone()));
	}

	unfound {
		let founder = T::SuspensionJudgementOrigin::successful_origin();
	}: unfound<T::Origin>(founder)
	verify {
		assert_eq!(<Founder<T, I>>::get(), None);
	}

	judge_suspended_member {
		let caller: T::AccountId = whitelisted_caller();
		let judgement_origin = T::SuspensionJudgementOrigin::successful_origin();
		Society::<T, I>::add_member(&caller)?;
		Society::<T, I>::suspend_member(&caller);
	}: judge_suspended_member<T::Origin>(judgement_origin, caller.clone(), false)
	verify {
		assert_eq!(<SuspendedMembers<T, I>>::contains_key(&caller), false);
	}

	judge_suspended_candidate {
		let caller: T::AccountId = whitelisted_caller();
		let judgement_origin = T::SuspensionJudgementOrigin::successful_origin();

		let mut pot = <Pot<T, I>>::get();
		pot = pot.saturating_add(BalanceOf::<T, I>::max_value());
		<Pot<T, I>>::put(&pot);

		let v: BalanceOf::<T, I> = 1u32.into();
		<SuspendedCandidates<T, I>>::insert(&caller, (v, BidKind::Deposit(1u32.into())));
	}: judge_suspended_candidate<T::Origin>(judgement_origin, caller.clone(), Judgement::Approve)
	verify {
		assert_eq!(<SuspendedCandidates<T, I>>::contains_key(&caller), false);
	}

	set_max_members {
		let root_account = RawOrigin::Root;
	}: set_max_members(root_account, 50)
	verify {
		assert_eq!(MaxMembers::<T, I>::get(), 50);
	}

	impl_benchmark_test_suite!(
		Society,
		crate::tests_composite::ExtBuilder::default().build(),
		crate::tests_composite::Test,
	)
}
