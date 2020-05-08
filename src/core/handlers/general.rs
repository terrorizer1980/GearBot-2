use std::sync::Arc;

use log::debug;
use twilight::gateway::cluster::Event;
use twilight::model::gateway::{
    payload::UpdateStatus,
    presence::{Activity, ActivityType, Status},
};

use crate::core::Context;
use crate::utils::Error;
use crate::{gearbot_info, gearbot_warn};

pub async fn handle_event(shard_id: u64, event: &Event, ctx: Arc<Context>) -> Result<(), Error> {
    match &event {
        Event::ShardReconnecting(_) => {
            gearbot_info!("Shard {} is attempting to reconnect", shard_id)
        }
        Event::ShardResuming(_) => gearbot_info!("Shard {} is resuming", shard_id),
        Event::Ready(_) => {
            gearbot_info!("Shard {} ready to go!", shard_id);
            ctx.cluster
                .command(
                    shard_id,
                    &UpdateStatus::new(
                        false,
                        gen_activity(String::from("the gears turn")),
                        None,
                        Status::Online,
                    ),
                )
                .await?;
        }
        Event::GatewayInvalidateSession(recon) => {
            if *recon {
                gearbot_warn!("The gateway has invalidated our session, but it is reconnectable!");
            } else {
                return Err(Error::InvalidSession);
            }
        }
        Event::GatewayReconnect => {
            gearbot_info!("Gateway requested shard {} to reconnect!", shard_id)
        }
        Event::GatewayHello(u) => {
            debug!("Registered with gateway {} on shard {}", u, shard_id);
            ctx.cluster
                .command(
                    shard_id,
                    &UpdateStatus::new(
                        true,
                        gen_activity(String::from("things coming online")),
                        None,
                        Status::Idle,
                    ),
                )
                .await?;
        }
        Event::Resumed => gearbot_info!("Shard {} successfully resumed", shard_id),
        Event::MemberChunk(chunk) => {
            debug!("got a chunk with nonce {:?}", &chunk.nonce);
            match &chunk.nonce {
                Some(nonce) => {
                    debug!("waiter found: {}", ctx.chunk_requests.contains_key(nonce));
                    match ctx.chunk_requests.remove(nonce) {
                        Some(waiter) => {
                            waiter.1.send(chunk.clone()).expect("Something went wrong when trying to forward a member chunk to it's receiver");
                        }
                        None => {}
                    }
                }
                None => {}
            };
        }
        _ => (),
    }
    Ok(())
}

fn gen_activity(name: String) -> Activity {
    Activity {
        assets: None,
        application_id: None,
        created_at: None,
        details: None,
        flags: None,
        id: None,
        instance: None,
        kind: ActivityType::Watching,
        name,
        emoji: None,
        party: None,
        secrets: None,
        state: None,
        timestamps: None,
        url: None,
    }
}
