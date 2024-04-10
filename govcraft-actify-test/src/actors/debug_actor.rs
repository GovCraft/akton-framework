use govcraft_actify::prelude::*;
use crate::MyMsg;

#[govcraft_actor("self::MyMsg")]
struct DebugActor {
    name: String,
}
#[govcraft_async]
impl GovcraftActor for DebugActor {
    type T = MyMsg;

    async fn handle_message(&mut self, message: Self::T, remaining: usize)-> anyhow::Result<()>{
        match message {
            MyMsg::Whisper(msg) => {
                println!("whisper for actor {} with value {} and {} remaining messages", self.name, msg, remaining);
            }
            _ => {}
        }
        Ok(())
    }
}
