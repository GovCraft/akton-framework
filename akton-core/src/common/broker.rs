/*
 *
 *  *
 *  * Copyright (c) 2024 Govcraft.
 *  *
 *  *  Licensed under the Business Source License, Version 1.1 (the "License");
 *  *  you may not use this file except in compliance with the License.
 *  *  You may obtain a copy of the License at
 *  *
 *  *      https://github.com/GovCraft/akton-framework/tree/main/LICENSES
 *  *
 *  *  Change Date: Three years from the release date of this version of the Licensed Work.
 *  *  Change License: Apache License, Version 2.0
 *  *
 *  *  Usage Limitations:
 *  *    - You may use the Licensed Work for non-production purposes only, such as internal testing, development, and experimentation.
 *  *    - You may not use the Licensed Work for any production or commercial purpose, including, but not limited to, the provision of any service to third parties, without a commercial use license from the Licensor, except as stated in the Exemptions section of the License.
 *  *
 *  *  Exemptions:
 *  *    - Open Source Projects licensed under an OSI-approved open source license.
 *  *    - Non-Profit Organizations using the Licensed Work for non-commercial purposes.
 *  *    - Small For-Profit Companies with annual gross revenues not exceeding $2,000,000 USD.
 *  *
 *  *  Unless required by applicable law or agreed to in writing, software
 *  *  distributed under the License is distributed on an "AS IS" BASIS,
 *  *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  *  See the License for the specific language governing permissions and
 *  *  limitations under the License.
 *  *
 *
 *
 */

use dashmap::DashMap;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::*;
use crate::actors::ActorConfig;
use crate::common::{Akton, Context};
use crate::message::{BroadcastEnvelope, SubscribeBroker, UnsubscribeBroker};
use crate::traits::{ActorContext, AktonMessage};


#[derive(Default, Debug)]
pub struct Broker {
    subscribers: Arc<DashMap<TypeId, HashSet<(String, Context)>>>,
}

impl Broker {
    #[instrument]
    pub async fn init() -> anyhow::Result<Context> {
        let actor_config = ActorConfig::new("broker", None, None);
        let mut actor = Akton::<Broker>::create_with_config(actor_config);

        actor.setup
            .act_on_async::<SubscribeBroker>(|actor, event| {
                let type_id = event.message.message_type_id.clone();
                let subscriber_context = event.message.subscriber_context.clone();
                let subscriber_id = event.message.subscriber_id.clone();

                // Event: Subscriber Added
                // Description: Triggered when a new subscriber is added.
                // Context: Includes type ID, subscriber context key, and subscriber ID.

                let subscribers = actor.state.subscribers.clone();
                Box::pin(async move {
                    subscribers
                        .entry(type_id)
                        .or_insert_with(HashSet::new)
                        .insert((subscriber_id.clone(), subscriber_context.clone()));
                    info!(type_id=?type_id, subscriber=subscriber_context.key.value, "Subscriber added");
                })
            });

        // Event: BrokerActor Activation
        // Description: Triggered when the BrokerActor is activated.
        // Context: None.
        info!("Activating the BrokerActor.");
        Ok(actor.activate(None).await?)
    }
    async fn emit_message_internal<M>(
        &self,
        subscriber_context: &Context,
        message: M,
    )
    where
        M: AktonMessage + Send + Sync,
    {
        subscriber_context.emit_async(message, None).await;
    }

    // pub async fn broadcast(subscribers: Arc<DashMap<TypeId, HashSet<(String, Context)>>>, message: BroadcastEnvelope) {
    //     let type_id = message.as_any().type_id();
    //     if let Some(subscribers) = subscribers.get(&type_id) {
    //         let mut futures = FuturesUnordered::new();
    //
    //         for (_, subscriber_context) in subscribers.value().clone() {
    //             let subscriber_context = subscriber_context.clone();
    //             let message = message.clone();
    //
    //             futures.push(async move {
    //                 subscriber_context.emit_async(message, None).await;
    //             });
    //         }
    //
    //         Self::broadcast_futures(futures);
    //     }
    // }
    //
    // fn broadcast_futures<T>(
    //     mut futures: FuturesUnordered<impl Future<Output=T> + Sized>,
    // ) -> Pin<Box<impl Future<Output=()> + Sized>> {
    //     trace!(
    //         futures_count = futures.len(),
    //         "Broadcasting futures to be processed."
    //     );
    //
    //     Box::pin(async move {
    //         while futures.next().await.is_some() {}
    //         trace!("All futures have been processed.");
    //     })
    // }
}
