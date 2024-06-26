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

use std::fmt::Display;

use akton_arn::{Arn, ArnParser};

use crate::common::{BrokerContext, ParentContext};

#[derive(Default, Debug, Clone)]
pub struct ActorConfig {
    id: String,
    broker: Option<BrokerContext>,
    parent: Option<ParentContext>,
}

impl ActorConfig {
    pub fn new(name: Arn<'_>, parent: Option<ParentContext>, broker: Option<BrokerContext>) -> anyhow::Result<ActorConfig> {
        if let Some(parent) = parent {
            //get the parent arn
            let parent_arn = ArnParser::new(&parent.key).parse()?;
            let child_arn = parent_arn + name;
            Ok(ActorConfig {
                id: child_arn.to_string(),
                broker,
                parent: Some(parent),
            })
        } else {
            Ok(ActorConfig {
                id: name.to_string(),
                broker,
                parent,
            })
        }
    }
    pub(crate) fn name(&self) -> &String {
        &self.id
    }
    pub(crate) fn get_broker(&self) -> &Option<BrokerContext> {
        &self.broker
    }
    pub(crate) fn parent(&self) -> &Option<ParentContext> {
        &self.parent
    }
}