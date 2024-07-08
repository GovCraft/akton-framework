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

mod idle;
pub mod running;

use std::any::{type_name_of_val, TypeId};
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use akton_arn::Arn;
use dashmap::DashMap;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::timeout;
use tokio_util::task::TaskTracker;
use tracing::*;
pub use idle::Idle;

use crate::common::{ActorRef, Acton, AktonInner, AsyncLifecycleHandler, BrokerRef, FutureBox, HaltSignal, IdleLifecycleHandler, LifecycleHandler, MessageHandler, ParentRef, ReactorItem, ReactorMap, SystemSignal};
use crate::message::{BrokerRequestEnvelope, Envelope, EventRecord, OutboundEnvelope};
use crate::prelude::{AktonMessage, SystemReady};
use crate::traits::Actor;

use super::{ActorConfig, Running};

pub struct ManagedActor<ActorState, ManagedEntity: Default + Send + Debug + 'static> {
    pub actor_ref: ActorRef,

    pub parent: Option<ParentRef>,

    pub broker: BrokerRef,

    pub halt_signal: HaltSignal,

    pub key: String,
    pub akton: SystemReady,

    pub entity: ManagedEntity,

    pub(crate) tracker: TaskTracker,

    pub inbox: Receiver<Envelope>,
    /// Reactor called before the actor wakes up.
    pub(crate) before_activate: Box<IdleLifecycleHandler<Idle, ManagedEntity>>,
    /// Reactor called when the actor wakes up.
    pub(crate) on_activate: Box<LifecycleHandler<Running, ManagedEntity>>,
    /// Reactor called just before the actor stops.
    pub(crate) before_stop: Box<LifecycleHandler<Running, ManagedEntity>>,
    /// Reactor called when the actor stops.
    pub(crate) on_stop: Box<LifecycleHandler<Running, ManagedEntity>>,
    /// Asynchronous reactor called just before the actor stops.
    pub(crate) before_stop_async: Option<AsyncLifecycleHandler<ManagedEntity>>,
    /// Map of reactors for handling different message types.
    pub(crate) reactors: ReactorMap<ManagedEntity>,
    _actor_state: std::marker::PhantomData<ActorState>,

}



impl<ActorState, ManagedEntity: Default + Send + Debug + 'static> Debug for ManagedActor<ActorState, ManagedEntity> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagedActor")
            .field("key", &self.key)
            .finish()
    }
}

// Function to downcast the message to the original type.
pub fn downcast_message<T: 'static>(msg: &dyn AktonMessage) -> Option<&T> {
    msg.as_any().downcast_ref::<T>()
}
