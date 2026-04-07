// mqtt.rs

use esp_idf_svc::mqtt::{
    self,
    client::{EventPayload, MessageId},
};
use esp_idf_sys::EspError;

use crate::*;

pub async fn run_mqtt(state: Arc<Pin<Box<MyState>>>) -> anyhow::Result<()> {
    if state.ap_mode {
        info!("MQTT is disabled in AP mode.");
        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    }

    if !state.config.mqtt_enable {
        info!("MQTT is disabled.");
        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    }

    loop {
        if *state.wifi_up.read().await {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    loop {
        let url = state.config.mqtt_url.clone();
        let myid = state.myid.read().await.clone();
        sleep(Duration::from_secs(5)).await;

        info!("MQTT conn: {url} [{myid}]");
        let (client, conn) = match mqtt::client::EspAsyncMqttClient::new(
            &url,
            &mqtt::client::MqttClientConfiguration {
                client_id: Some(&myid),
                keep_alive_interval: Some(Duration::from_secs(25)),
                ..Default::default()
            },
        ) {
            Ok(c) => c,
            Err(e) => {
                error!("MQTT conn failed: {e:?}");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let _ = tokio::try_join!(
            Box::pin(subscribe_publish(state.clone(), client)),
            Box::pin(event_loop(state.clone(), conn)),
        );

        error!("MQTT connection dropped, retrying in 30 s.");
        sleep(Duration::from_secs(30)).await;
    }
}

async fn subscribe_publish(
    state: Arc<Pin<Box<MyState>>>,
    mut client: mqtt::client::EspAsyncMqttClient,
) -> anyhow::Result<()> {
    let base = state.config.mqtt_topic.clone();
    let cmd_sample = format!("{base}/cmd/sample");
    let cmd_led = format!("{base}/cmd/led");
    let cmd_reboot = format!("{base}/cmd/reboot");

    for topic in [&cmd_sample, &cmd_led, &cmd_reboot] {
        info!("MQTT subscribe {topic}");
        client
            .subscribe(topic, mqtt::client::QoS::AtLeastOnce)
            .await?;
        sleep(Duration::from_millis(250)).await;
    }

    let mut last_status_publish = 0u32;

    loop {
        sleep(Duration::from_secs(5)).await;
        let uptime = state.data.read().await.uptime;

        if uptime >= last_status_publish.saturating_add(60) {
            last_status_publish = uptime;
            Box::pin(publish_status(&state, &mut client)).await?;
        }

        {
            let mut fresh_data = state.fresh_data.write().await;
            if *fresh_data {
                *fresh_data = false;
                Box::pin(publish_sensor_data(&state, &base, &mut client, uptime)).await?;
            }
        }

        {
            let mut sample_updated = state.sample_updated.write().await;
            if *sample_updated {
                *sample_updated = false;
                publish_sample(&state, &base, &mut client).await?;
            }
        }
    }
}

async fn publish_status(
    state: &Arc<Pin<Box<MyState>>>,
    client: &mut mqtt::client::EspAsyncMqttClient,
) -> anyhow::Result<()> {
    let uptime = state.data.read().await.uptime;
    let ip_addr = state.ip_addr.read().await.to_string();
    let myid = state.myid.read().await.clone();
    let topic = format!("{}/state", state.config.mqtt_topic);
    let data = serde_json::json!({
        "id": myid,
        "ap_mode": state.ap_mode,
        "wifi_up": *state.wifi_up.read().await,
        "ntp_ok": *state.ntp_ok.read().await,
        "ip_addr": ip_addr,
        "uptime": uptime,
    })
    .to_string();
    Box::pin(mqtt_send(client, &topic, &data)).await?;
    Ok(())
}

async fn publish_sensor_data(
    state: &Arc<Pin<Box<MyState>>>,
    base: &str,
    client: &mut mqtt::client::EspAsyncMqttClient,
    uptime: u32,
) -> anyhow::Result<()> {
    let topic = format!("{base}/uptime");
    let data = format!("{{ \"uptime\": {uptime} }}");
    Box::pin(mqtt_send(client, &topic, &data)).await?;

    let snapshot = state.data.read().await.clone();
    for value in snapshot.temperatures.iter().filter(|v| v.value > NO_TEMP) {
        let topic = format!("{base}/sensor/{}", value.sensor);
        let data = serde_json::json!({
            "iopin": value.iopin,
            "sensor": value.sensor,
            "temperature": value.value,
            "timestamp": snapshot.timestamp,
        })
        .to_string();
        Box::pin(mqtt_send(client, &topic, &data)).await?;
    }

    Ok(())
}

async fn publish_sample(
    state: &Arc<Pin<Box<MyState>>>,
    base: &str,
    client: &mut mqtt::client::EspAsyncMqttClient,
) -> anyhow::Result<()> {
    let topic = format!("{base}/sample");
    let data = serde_json::to_string(&*state.sample.read().await)?;
    Box::pin(mqtt_send(client, &topic, &data)).await?;
    Ok(())
}

async fn mqtt_send(
    client: &mut mqtt::client::EspAsyncMqttClient,
    topic: &str,
    data: &str,
) -> Result<MessageId, EspError> {
    info!("MQTT sending {topic} {data}");

    let result = client
        .publish(
            topic,
            mqtt::client::QoS::AtLeastOnce,
            false,
            data.as_bytes(),
        )
        .await;
    if let Err(e) = result {
        let msg = format!("MQTT send error: {e}");
        error!("{msg}");
    }
    result
}

async fn event_loop(
    state: Arc<Pin<Box<MyState>>>,
    mut conn: mqtt::client::EspAsyncMqttConnection,
) -> anyhow::Result<()> {
    while let Ok(notification) = Box::pin(conn.next()).await {
        if let EventPayload::Received {
            topic: Some(topic),
            data,
            ..
        } = notification.payload()
        {
            let base = &state.config.mqtt_topic;

            if topic == format!("{base}/cmd/sample") {
                match serde_json::from_slice::<SampleMessage>(data) {
                    Ok(msg) => state.update_sample_message("mqtt", msg.message).await,
                    Err(e) => error!("MQTT sample command JSON error: {e}"),
                }
                continue;
            }

            if topic == format!("{base}/cmd/led") {
                match serde_json::from_slice::<LedCommand>(data) {
                    Ok(cmd) => {
                        if let Err(e) = state.set_led(cmd.on).await {
                            error!("MQTT led command error: {e:#}");
                        } else if let Some(duration_ms) = cmd.duration_ms {
                            sleep(Duration::from_millis(duration_ms)).await;
                            state.led_off().await.ok();
                        }
                    }
                    Err(e) => error!("MQTT led command JSON error: {e}"),
                }
                continue;
            }

            if topic == format!("{base}/cmd/reboot") {
                state
                    .update_sample_message("mqtt", "reboot requested over MQTT")
                    .await;
                *state.reset.write().await = true;
                continue;
            }
        }
    }

    error!("MQTT connection closed.");
    Ok(())
}
