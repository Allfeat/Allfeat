// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as AtsPallet;

use frame_benchmarking::v2::*;
use frame_support::traits::fungible::Mutate;
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
    use super::*;
    use frame_support::sp_runtime::traits::Bounded;

    #[benchmark]
    fn register(x: Linear<32, 1024>) {
        let provider: T::AccountId = whitelisted_caller();
        let _ = T::Currency::set_balance(&provider, init_bal::<T>());

        // Create valid ZKP test data (using the valid proof from tests)
        let vk = hex::decode("0dad748a7ef4a81fc022070d1d92142ce7dfe8565c4852fcd25fbcaf7906759f9b724b742bb839ebb319eb346ed517faf82e7e66276219c37cfa8d9d0b5cef0a3494b9dfc76fe7406f71be3fe5c2a72f04d85d13d113f0d2926f9b44f7876a29cf9f3060f8e58114518eb3d3ae033419dca17765b66b106dac5647cabc47da13169bc4a3e626f2200ba189f9f17f548cb66d6ef1da7c3e9db81388ae2f834d895d789bb4d21c35d8257991b0a339bbbd488328a7ee70358265b35bb3181aef02899afeaf67b693aa04828aa929998d0152527f2e67f901fab54f8717709e9faa0700000000000000e9e1273293c1a32aa27705729bb1f2e0293e1cb744a087c70d369d25cddff2a4ef73d88aec5f058ac2de61635a380211e49276e772c7926edb5264069101b106c91a5c9405a7b26c9bc188cd29d1275b141fdda0d766fbf019c2563b73c6d8ae2f6652677d17fc5f2e9c49ede6df9b01fe3ed1992a50c0d7c645a1852ce68f197fb033f9073337dbdf7645ad8efe51b9cbacb4726984a41fa00fadf2f73a080bf528732cf871bcc682a10d6a5973464b35e8589fe33a37d08748f8e4adc4470c60d97cbb85e99ff481168bda0d45c68e10a7433cea5287523ec800292cf94c95").unwrap();
        let proof = hex::decode("2e2008dc99bbc214438279dc6c527abf5d3b544d6535e2e1a8240eff60e3528524009ffa9f7dd9582f4aea6d64ee999dcbc068d84293f15ab7ee8121d4b5e812970acdff96b8371b2b75a194f591a0cb5c104aef6ad3523376f11cf17e13f7af3ba5ca7ff69cd5262c34092becafc3e44df7be4a830388640d8fd1821687d3a4").unwrap();

        let pubs: Vec<[u8; 32]> = vec![
            hex::decode("26d273f7c73a635f6eaeb904e116ec4cd887fb5a87fc7427c95279e6053e5bf0")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("175eeef716d52cf8ee972c6fefd60e47df5084efde3c188c40a81a42e72dfb04")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("017ac5e7a52bec07ca8ee344a9979aa083b7713f1196af35310de21746985079")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("2a6dda925d7af47190183415517709278c73a94b40ab39f56d058c0bf0a84c68")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000002710")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("17c57750af41a2dc524ba01dd95bf7876d738eac80936fe96f374086ed91391d")
                .unwrap()
                .try_into()
                .unwrap(),
        ];

        #[extrinsic_call]
        _(RawOrigin::Signed(provider.clone()), vk, pubs.clone(), proof);

        let hash_commitment = pubs[3];
        assert!(AtsOf::<T>::get(hash_commitment).is_some());
    }

    #[benchmark]
    fn claim() -> Result<(), BenchmarkError> {
        let original_owner: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("claimer", 0, 0);

        let _ = T::Currency::set_balance(&original_owner, init_bal::<T>());
        let _ = T::Currency::set_balance(&new_owner, init_bal::<T>());

        // Register first
        let vk = hex::decode("0dad748a7ef4a81fc022070d1d92142ce7dfe8565c4852fcd25fbcaf7906759f9b724b742bb839ebb319eb346ed517faf82e7e66276219c37cfa8d9d0b5cef0a3494b9dfc76fe7406f71be3fe5c2a72f04d85d13d113f0d2926f9b44f7876a29cf9f3060f8e58114518eb3d3ae033419dca17765b66b106dac5647cabc47da13169bc4a3e626f2200ba189f9f17f548cb66d6ef1da7c3e9db81388ae2f834d895d789bb4d21c35d8257991b0a339bbbd488328a7ee70358265b35bb3181aef02899afeaf67b693aa04828aa929998d0152527f2e67f901fab54f8717709e9faa0700000000000000e9e1273293c1a32aa27705729bb1f2e0293e1cb744a087c70d369d25cddff2a4ef73d88aec5f058ac2de61635a380211e49276e772c7926edb5264069101b106c91a5c9405a7b26c9bc188cd29d1275b141fdda0d766fbf019c2563b73c6d8ae2f6652677d17fc5f2e9c49ede6df9b01fe3ed1992a50c0d7c645a1852ce68f197fb033f9073337dbdf7645ad8efe51b9cbacb4726984a41fa00fadf2f73a080bf528732cf871bcc682a10d6a5973464b35e8589fe33a37d08748f8e4adc4470c60d97cbb85e99ff481168bda0d45c68e10a7433cea5287523ec800292cf94c95").unwrap();
        let proof = hex::decode("2e2008dc99bbc214438279dc6c527abf5d3b544d6535e2e1a8240eff60e3528524009ffa9f7dd9582f4aea6d64ee999dcbc068d84293f15ab7ee8121d4b5e812970acdff96b8371b2b75a194f591a0cb5c104aef6ad3523376f11cf17e13f7af3ba5ca7ff69cd5262c34092becafc3e44df7be4a830388640d8fd1821687d3a4").unwrap();

        let pubs: Vec<[u8; 32]> = vec![
            hex::decode("26d273f7c73a635f6eaeb904e116ec4cd887fb5a87fc7427c95279e6053e5bf0")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("175eeef716d52cf8ee972c6fefd60e47df5084efde3c188c40a81a42e72dfb04")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("017ac5e7a52bec07ca8ee344a9979aa083b7713f1196af35310de21746985079")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("2a6dda925d7af47190183415517709278c73a94b40ab39f56d058c0bf0a84c68")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("0000000000000000000000000000000000000000000000000000000000002710")
                .unwrap()
                .try_into()
                .unwrap(),
            hex::decode("17c57750af41a2dc524ba01dd95bf7876d738eac80936fe96f374086ed91391d")
                .unwrap()
                .try_into()
                .unwrap(),
        ];

        AtsPallet::<T>::register(
            RawOrigin::Signed(original_owner.clone()).into(),
            vk.clone(),
            pubs.clone(),
            proof.clone(),
        )?;

        let hash_commitment = pubs[3];

        #[extrinsic_call]
        _(RawOrigin::Signed(new_owner.clone()), vk, pubs, proof);

        assert_last_event::<T>(
            Event::ATSClaimed {
                old_owner: original_owner,
                new_owner,
                hash_commitment,
            }
            .into(),
        );
        Ok(())
    }

    fn init_bal<T: Config>() -> BalanceOf<T> {
        BalanceOf::<T>::max_value() / 10u32.into()
    }

    impl_benchmark_test_suite!(AtsPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
