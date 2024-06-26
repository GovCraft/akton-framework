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
use std::fmt::Debug;
use std::future::Future;
use std::hash::{Hash, Hasher};

use akton_arn::Arn;
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::oneshot;
use tokio_util::task::TaskTracker;
use tracing::{info, instrument, trace, warn};

use crate::actors::{Actor, Idle};
use crate::common::{BrokerContext, OutboundChannel, OutboundEnvelope, ParentContext, SystemSignal};
use crate::traits::{ActorContext, Subscriber};

/// Represents the context in which an actor operates.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// The unique identifier (ARN) for the context.
    pub key: String,
    /// The outbound channel for sending messages.
    pub(crate) outbox: Option<OutboundChannel>,
    /// The task tracker for the actor.
    pub(crate) task_tracker: TaskTracker,
    /// The actor's optional parent context.
    pub parent: Option<Box<ParentContext>>,
    pub broker: Box<Option<BrokerContext>>,
    pub(crate) children: DashMap<String, Context>,
}

impl Subscriber for Context {
    fn get_broker(&self) -> Option<BrokerContext> {
        *self.broker.clone()
    }
}

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Context {}

impl Hash for Context {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl Context {
    #[instrument(skip(self))]
    pub async fn supervise<State: Default + Send + Debug>(
        &self,
        child: Actor<Idle<State>, State>,
    ) -> anyhow::Result<()> {
        let context = child.activate(None).await;
        let id = context.key.clone();
        self.children.insert(id, context);

        Ok(())
    }
}



#[async_trait]
impl ActorContext for Context {
    /// Returns the return address for the actor.
    #[instrument(skip(self))]
    fn get_return_address(&self) -> OutboundEnvelope {
        let outbox = self.outbox.clone();
        OutboundEnvelope::new(outbox, self.key.clone())
    }
    // #[instrument(Level::TRACE, skip(self), fields(child_count = self.children.len()))]
    fn get_children(&self) -> DashMap<String, Context> {
        // event!(Level::TRACE,child_count= self.children.len());
        self.children.clone()
    }

    fn find_child_by_arn(&self, arn: &str) -> Option<Context> {
        self.children.get(arn).map(|item| item.value().clone())
    }

    /// Returns the task tracker for the actor.
    fn get_task_tracker(&self) -> TaskTracker {
        self.task_tracker.clone()
    }

    fn get_id(&self) -> String {
        self.key.clone()
    }

    fn clone_context(&self) -> Context {
        self.clone()
    }

    /// Wakes the actor.
    async fn wake_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Recreates the actor.
    async fn recreate_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Suspends the actor.
    fn suspend_actor(&self) -> impl Future<Output=anyhow::Result<()>> + Send + Sync + '_ {
        async move {
            let tracker = self.get_task_tracker().clone();

            let actor = self.get_return_address().clone();


            // Event: Sending Terminate Signal
            // Description: Sending a terminate signal to the actor.
            // Context: Target actor key.
            warn!(actor=self.key, "Sending Terminate to");
            actor.reply(SystemSignal::Terminate, None)?;

            // Event: Waiting for Actor Tasks
            // Description: Waiting for all actor tasks to complete.
            // Context: None
            trace!("Waiting for all actor tasks to complete.");
            tracker.wait().await;

            // Event: Actor Terminated
            // Description: The actor and its subordinates have been terminated.
            // Context: None
            info!(actor=self.key, "The actor and its subordinates have been terminated.");
            Ok(())
        }
    }

    /// Resumes the actor.
    async fn resume_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Supervises the actor.
    async fn supervise_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Watches the actor.
    async fn watch_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Stops watching the actor.
    async fn unwatch_actor(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }

    /// Marks the actor as failed.
    async fn mark_as_failed(&mut self) -> anyhow::Result<()> {
        unimplemented!()
    }
}
