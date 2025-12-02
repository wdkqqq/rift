// WIP / ON HOLD
// Will pick this up again in the near future.

use chrono::Utc;
use discord_rich_presence::activity::ActivityType;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::thread;
use std::time::Duration;

use crate::config::config::load_config;

pub fn discord_rpc() {
    let config = load_config();

    if !config.discord_rpc {
        return;
    }

    let mut client = DiscordIpcClient::new("1443346664822673538");

    if let Err(e) = client.connect() {
        eprintln!("Cannot connect to Discord: {}", e);
        return;
    }

    let timestamp = Utc::now().timestamp();

    let base_activity = || {
        activity::Activity::new()
            .activity_type(ActivityType::Listening)
            .details("NAME")
            .state("ARTIST")
            .assets(activity::Assets::new().large_image("logo"))
    };

    if let Err(e) = client.set_activity(
        base_activity().timestamps(activity::Timestamps::new().start(timestamp).end(timestamp)),
    ) {
        eprintln!("Cannot set activity: {}", e);
        return;
    }

    println!("Discord RPC activated successfully!");

    loop {
        thread::sleep(Duration::from_secs(30));

        let new_timestamp = Utc::now().timestamp();
        let _ = client.set_activity(
            base_activity().timestamps(activity::Timestamps::new().end(new_timestamp)),
        );
    }
}
