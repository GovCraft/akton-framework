/*
 *
 *  * Copyright (c) 2024 Govcraft.
 *  *
 *  * Licensed under the Apache License, Version 2.0 (the "License");
 *  * you may not use this file except in compliance with the License.
 *  * You may obtain a copy of the License at
 *  *
 *  *     http://www.apache.org/licenses/LICENSE-2.0
 *  *
 *  * Unless required by applicable law or agreed to in writing, software
 *  * distributed under the License is distributed on an "AS IS" BASIS,
 *  * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  * See the License for the specific language governing permissions and
 *  * limitations under the License.
 *
 *
 */

use std::any::TypeId;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use dashmap::DashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use crate::traits::{QuasarMessage, SystemMessage};
use crate::common::{Actor, Awake, Context, Envelope};

pub type ReactorMap<T> = DashMap<TypeId, ReactorItem<T>>;

pub enum ReactorItem<T: Default + Send + Debug + 'static> {
    Signal(Box<SignalReactor<T>>),
    Message(Box<MessageReactor<T>>),
    Future(Box<FutReactor<T>>),
}

pub type SignalReactor<State> = dyn for<'a, 'b> Fn(Actor<Awake<State>, State>, &dyn QuasarMessage) -> Pin<Box<dyn Future<Output=()> + Send + 'static>> + Send + 'static;
pub type MessageReactor<State> = dyn for<'a, 'b> Fn(&mut Actor<Awake<State>, State>, &'b Envelope) + Send + 'static;
pub type FutReactor<State> = dyn for<'a, 'b> Fn(&mut Actor<Awake<State>, State>, &'b Envelope) -> Fut + Send + 'static;


pub type Fut = Pin<Box<dyn Future<Output=()> + Send + 'static>>;
pub type BoxFutReactor<T> = Box<FutReactor<T>>;
pub type PinBoxFutReactor<T> = Pin<BoxFutReactor<T>>;


pub type OutboundChannel = Sender<Envelope>;
pub type InboundChannel = Receiver<Envelope>;
pub type StopSignal = AtomicBool;

pub type ContextPool = DashMap<String, Context>;
pub type ActorPool = DashMap<String, ContextPool>;
pub type LifecycleReactor<T, State> = dyn Fn(&Actor<T, State>) + Send;
pub type IdleLifecycleReactor<T, State> = dyn Fn(&Actor<T, State>) + Send;
