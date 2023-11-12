use glonax::SharedRuntimeState;

use crate::config::ProxyConfig;

const _REMOTE_PROBE_HOST: &str = "https://cymbion-oybqn.ondigitalocean.app";

pub(super) async fn service(_config: ProxyConfig, _runtime_state: SharedRuntimeState) {
    // log::debug!("Starting host service");

    // let url = reqwest::Url::parse(REMOTE_PROBE_HOST).unwrap();

    // let client = reqwest::Client::builder()
    //     .user_agent("glonax-agent/0.1.0")
    //     .timeout(std::time::Duration::from_secs(5))
    //     .https_only(true)
    //     .build()
    //     .unwrap();

    // let request_url = url
    //     .join(&format!("api/v1/{}/probe", config.instance.id))
    //     .unwrap();

    // loop {
    // tokio::time::sleep(std::time::Duration::from_secs(local_config.probe_interval)).await;

    // if config.probe {
    //     let data = telemetrics.read().await;

    //     if data.status.is_none() {
    //         continue;
    //     }

    //     let response = client
    //         .post(request_url.clone())
    //         .json(&*data)
    //         .send()
    //         .await
    //         .unwrap();

    //     if response.status() == 200 {
    //         log::info!("Probe sent successfully");
    //     } else {
    //         log::error!("Probe failed, status: {}", response.status());
    //     }
    // };

    // log::trace!("{}", local_machine_state.read().await.data);
    // }
}
