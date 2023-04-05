use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt as mqtt;
use std::{process, time::Duration};

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[1, 1];

#[macro_use]
extern crate dotenv_codegen;

fn main() {
    // Initialize the logger from the environment
    env_logger::init();

    let client_name: &str = dotenv!("CLIENT_NAME");

    // Create the client. Use a Client ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(dotenv!("MQTT_URL"))
        .client_id(client_name)
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(25);

        // Define the set of options for the connection
        let lwt = mqtt::Message::new("death/mori-summer", "VAN!SHMENT THIS WORLD", mqtt::QOS_1);
        let intro = mqtt::Message::new("hello/mori-summer", "Hello from Rust!", mqtt::QOS_1);

        // Create the connect options, explicitly requesting MQTT v3.x
        let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
            .keep_alive_interval(Duration::from_secs(5))
            .user_name(dotenv!("MQTT_USER"))
            .password(dotenv!("MQTT_PASSWORD"))
            .clean_session(false)
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        println!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        println!("Subscribing to topics: {:?}", TOPICS);
        cli.subscribe_many(TOPICS, QOS).await?;

        cli.publish(intro.clone()).await?;
        // Just loop on incoming messages.
        println!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                println!("{}", msg);
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    println!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }

                // Re-publish our Hello message
                cli.publish(intro.clone()).await?;
            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}
