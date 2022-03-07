// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{
	dollar, AccountId, Balance, Currencies, CurrencyId, Dex, Event, GetNativeCurrencyId, GetSetUSDId, Runtime,
	System, TradingPathLimit,
};

use frame_benchmarking::{account, whitelisted_caller};
use frame_system::RawOrigin;
use module_dex::TradingPairStatus;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::TradingPair;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::prelude::*;

const SEED: u32 = 0;

const NATIVE: CurrencyId = GetNativeCurrencyId::get();
const STABLECOIN: CurrencyId = GetSetUSDId::get();
const CURRENCY_LIST: [CurrencyId; 2] = [NATIVE, STABLECOIN];

fn assert_last_event(generic_event: Event) {
	System::assert_last_event(generic_event.into());
}

fn inject_liquidity(
	maker: AccountId,
	currency_id_a: CurrencyId,
	currency_id_b: CurrencyId,
	max_amount_a: Balance,
	max_amount_b: Balance,
) -> Result<(), &'static str> {
	// set balance
	<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id_a,
		&maker,
		max_amount_a.unique_saturated_into(),
	)?;
	<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id_b,
		&maker,
		max_amount_b.unique_saturated_into(),
	)?;

	let _ = Dex::enable_trading_pair(RawOrigin::Root.into(), currency_id_a, currency_id_b);

	Dex::add_liquidity(
		RawOrigin::Signed(maker.clone()).into(),
		currency_id_a,
		currency_id_b,
		max_amount_a,
		max_amount_b,
		Default::default(),
	)?;

	Ok(())
}

runtime_benchmarks! {
	{ Runtime, module_dex }

	// enable a Disabled trading pair
	enable_trading_pair {
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
	}: _(RawOrigin::Root, trading_pair.first(), trading_pair.second())
	verify {
		assert_last_event(module_dex::Event::EnableTradingPair{trading_pair: trading_pair}.into());
	}

	// disable a Enabled trading pair
	disable_trading_pair {
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Disabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::enable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
	}: _(RawOrigin::Root, trading_pair.first(), trading_pair.second())
	verify {
		assert_last_event(module_dex::Event::DisableTradingPair{trading_pair}.into());
	}

	// list a Provisioning trading pair
	list_provisioning {
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
	}: _(RawOrigin::Root, trading_pair.first(), trading_pair.second(), dollar(trading_pair.first()), dollar(trading_pair.second()), dollar(trading_pair.first()), dollar(trading_pair.second()), 10)
	verify {
		assert_last_event(module_dex::Event::ListProvisioning{trading_pair: trading_pair}.into());
	}

	// update parameters of a Provisioning trading pair
	update_provisioning_parameters {
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
		Dex::list_provisioning(
			RawOrigin::Root.into(),
			trading_pair.first(),
			trading_pair.second(),
			dollar(trading_pair.first()),
			dollar(trading_pair.second()),
			100 * dollar(trading_pair.first()),
			1000 * dollar(trading_pair.second()),
			100
		)?;
	}: _(RawOrigin::Root, trading_pair.first(), trading_pair.second(), 2 * dollar(trading_pair.first()), 2 * dollar(trading_pair.second()), 10 * dollar(trading_pair.first()), 100 * dollar(trading_pair.second()), 200)

	// end a Provisioning trading pair
	end_provisioning {
		let founder: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
		Dex::list_provisioning(
			RawOrigin::Root.into(),
			trading_pair.first(),
			trading_pair.second(),
			dollar(trading_pair.first()),
			dollar(trading_pair.second()),
			100 * dollar(trading_pair.first()),
			100 * dollar(trading_pair.second()),
			0
		)?;

		// set balance
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.first(), &founder, (100 * dollar(trading_pair.first())).unique_saturated_into())?;
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.second(), &founder, (100 * dollar(trading_pair.second())).unique_saturated_into())?;

		// add enough provision
		Dex::add_provision(
			RawOrigin::Signed(founder.clone()).into(),
			trading_pair.first(),
			trading_pair.second(),
			100 * dollar(trading_pair.first()),
			100 * dollar(trading_pair.second()),
		)?;
	}: _(RawOrigin::Signed(founder), trading_pair.first(), trading_pair.second())
	verify {
		assert_last_event(module_dex::Event::ProvisioningToEnabled{trading_pair, pool_0: 100 * dollar(trading_pair.first()), pool_1: 100 * dollar(trading_pair.second()), share_amount: 200 * dollar(trading_pair.first())}.into())
	}

	add_provision {
		let founder: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
		Dex::list_provisioning(
			RawOrigin::Root.into(),
			trading_pair.first(),
			trading_pair.second(),
			dollar(trading_pair.first()),
			dollar(trading_pair.second()),
			100 * dollar(trading_pair.first()),
			1000 * dollar(trading_pair.second()),
			0
		)?;

		// set balance
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.first(), &founder, (10 * dollar(trading_pair.first())).unique_saturated_into())?;
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.second(), &founder, (10 * dollar(trading_pair.second())).unique_saturated_into())?;
	}: _(RawOrigin::Signed(founder.clone()), trading_pair.first(), trading_pair.second(), dollar(trading_pair.first()), dollar(trading_pair.second()))
	verify{
		assert_last_event(module_dex::Event::AddProvision{who: founder, currency_0: trading_pair.first(), contribution_0: dollar(trading_pair.first()), currency_1: trading_pair.second(), contribution_1: dollar(trading_pair.second())}.into());
	}

	claim_dex_share {
		let founder: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		if let TradingPairStatus::Enabled = Dex::trading_pair_statuses(trading_pair) {
			Dex::disable_trading_pair(RawOrigin::Root.into(), trading_pair.first(), trading_pair.second())?;
		}
		Dex::list_provisioning(
			RawOrigin::Root.into(),
			trading_pair.first(),
			trading_pair.second(),
			dollar(trading_pair.first()),
			dollar(trading_pair.second()),
			10 * dollar(trading_pair.first()),
			10 * dollar(trading_pair.second()),
			0
		)?;

		// set balance
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.first(), &founder, (100 * dollar(trading_pair.first())).unique_saturated_into())?;
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.second(), &founder, (100 * dollar(trading_pair.second())).unique_saturated_into())?;

		Dex::add_provision(
			RawOrigin::Signed(founder.clone()).into(),
			trading_pair.first(),
			trading_pair.second(),
			dollar(trading_pair.first()),
			20 * dollar(trading_pair.second())
		)?;
		Dex::end_provisioning(
			RawOrigin::Signed(founder.clone()).into(),
			trading_pair.first(),
			trading_pair.second(),
		)?;
	}: _(RawOrigin::Signed(whitelisted_caller()), founder.clone(), trading_pair.first(), trading_pair.second())
	verify {
		assert_eq!(Currencies::free_balance(trading_pair.dex_share_currency_id(), &founder), 2_000_000_000_000);
	}

	// add liquidity
	add_liquidity {
		let first_maker: AccountId = account("first_maker", 0, SEED);
		let second_maker: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		let amount_a = 100 * dollar(trading_pair.first());
		let amount_b = 10_000 * dollar(trading_pair.second());

		// set balance
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.first(), &second_maker, amount_a.unique_saturated_into())?;
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.second(), &second_maker, amount_b.unique_saturated_into())?;

		// first maker inject liquidity
		inject_liquidity(first_maker.clone(), trading_pair.first(), trading_pair.second(), amount_a, amount_b)?;
	}: add_liquidity(RawOrigin::Signed(second_maker), trading_pair.first(), trading_pair.second(), amount_a, amount_b, Default::default())

	// worst: add liquidity
	add_liquidity_and_stake {
		let first_maker: AccountId = account("first_maker", 0, SEED);
		let second_maker: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		let amount_a = 100 * dollar(trading_pair.first());
		let amount_b = 10_000 * dollar(trading_pair.second());

		// set balance
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.first(), &second_maker, amount_a.unique_saturated_into())?;
		<Currencies as MultiCurrencyExtended<_>>::update_balance(trading_pair.second(), &second_maker, amount_b.unique_saturated_into())?;

		// first maker inject liquidity
		inject_liquidity(first_maker.clone(), trading_pair.first(), trading_pair.second(), amount_a, amount_b)?;
	}: add_liquidity(RawOrigin::Signed(second_maker), trading_pair.first(), trading_pair.second(), amount_a, amount_b, Default::default())

	// remove liquidity by liquid lp share
	remove_liquidity {
		let maker: AccountId = whitelisted_caller();
		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		inject_liquidity(maker.clone(), trading_pair.first(), trading_pair.second(), 100 * dollar(trading_pair.first()), 10_000 * dollar(trading_pair.second()))?;
	}: remove_liquidity(RawOrigin::Signed(maker), trading_pair.first(), trading_pair.second(), 50 * dollar(trading_pair.first()), Default::default(), Default::default())

	swap_with_exact_supply {
		let u in 2 .. TradingPathLimit::get() as u32;

		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		let mut path: Vec<CurrencyId> = vec![];
		for i in 1 .. u {
			if i == 1 {
				path.push(trading_pair.first());
				path.push(trading_pair.second());
			} else {
				if i % 2 == 0 {
					path.push(trading_pair.first());
				} else {
					path.push(trading_pair.second());
				}
			}
		}

		let maker: AccountId = account("maker", 0, SEED);
		let taker: AccountId = whitelisted_caller();
		inject_liquidity(maker, trading_pair.first(), trading_pair.second(), 10_000 * dollar(trading_pair.first()), 10_000 * dollar(trading_pair.second()))?;

		<Currencies as MultiCurrencyExtended<_>>::update_balance(path[0], &taker, (10_000 * dollar(path[0])).unique_saturated_into())?;
	}: swap_with_exact_supply(RawOrigin::Signed(taker), path.clone(), 100 * dollar(path[0]), 0)
	verify {
		// would panic the benchmark anyways, must add new currencies to CURRENCY_LIST for benchmarking to work
		assert!(TradingPathLimit::get() < CURRENCY_LIST.len() as u32);
	}

	swap_with_exact_target {
		let u in 2 .. TradingPathLimit::get() as u32;

		let trading_pair = TradingPair::from_currency_ids(STABLECOIN, NATIVE).unwrap();
		let mut path: Vec<CurrencyId> = vec![];
		for i in 1 .. u {
			if i == 1 {
				path.push(trading_pair.first());
				path.push(trading_pair.second());
			} else {
				if i % 2 == 0 {
					path.push(trading_pair.first());
				} else {
					path.push(trading_pair.second());
				}
			}
		}

		let maker: AccountId = account("maker", 0, SEED);
		let taker: AccountId = whitelisted_caller();
		inject_liquidity(maker, trading_pair.first(), trading_pair.second(), 10_000 * dollar(trading_pair.first()), 10_000 * dollar(trading_pair.second()))?;

		<Currencies as MultiCurrencyExtended<_>>::update_balance(path[0], &taker, (10_000 * dollar(path[0])).unique_saturated_into())?;
	}: swap_with_exact_target(RawOrigin::Signed(taker), path.clone(), 10 * dollar(path[path.len() - 1]), 100 * dollar(path[0]))
	verify {
		// would panic the benchmark anyways, must add new currencies to CURRENCY_LIST for benchmarking to work
		assert!(TradingPathLimit::get() < CURRENCY_LIST.len() as u32);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
