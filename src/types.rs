/*
 * Copyright 2025 Flamewire
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use scale_value::Composite;
use subxt::events::EventDetails;
use subxt::Config;

pub type BlockNumber = u64;

pub struct ChainEvent<C: Config> {
    inner: EventDetails<C>,
}

impl<C: Config> ChainEvent<C> {
    pub fn new(inner: EventDetails<C>) -> Self {
        Self { inner }
    }

    pub fn pallet_name(&self) -> &str {
        self.inner.pallet_name()
    }

    pub fn variant_name(&self) -> &str {
        self.inner.variant_name()
    }

    pub fn as_event<T: subxt::events::StaticEvent + 'static>(
        &self,
    ) -> Result<Option<T>, Box<subxt::Error>> {
        self.inner.as_event::<T>().map_err(Box::new)
    }

    pub fn field_values(&self) -> Result<Composite<u32>, Box<subxt::Error>> {
        self.inner.field_values().map_err(Box::new)
    }
}
