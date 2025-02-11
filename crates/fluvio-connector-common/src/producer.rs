use fluvio::{FluvioConfig, Fluvio, TopicProducer, TopicProducerConfigBuilder};
use crate::{ConnectorConfig, Result};

use crate::{ensure_topic_exists, smartmodule::smartmodule_chain_from_config};

pub async fn producer_from_config(config: &ConnectorConfig) -> Result<(Fluvio, TopicProducer)> {
    let mut cluster_config = FluvioConfig::load()?;
    cluster_config.client_id = Some(format!("fluvio_connector_{}", &config.name));

    let fluvio = Fluvio::connect_with_config(&cluster_config).await?;
    ensure_topic_exists(config).await?;
    let mut config_builder = TopicProducerConfigBuilder::default();

    if let Some(producer_params) = &config.producer {
        // Linger
        if let Some(linger) = producer_params.linger {
            config_builder = config_builder.linger(linger)
        };

        // Compression
        if let Some(compression) = producer_params.compression {
            config_builder = config_builder.compression(compression)
        };

        // Batch size
        if let Some(batch_size) = producer_params.batch_size {
            config_builder = config_builder.batch_size(batch_size.as_u64() as usize)
        };
    };

    let producer_config = config_builder.build()?;
    let producer = fluvio
        .topic_producer_with_config(&config.topic, producer_config)
        .await?;

    if let Some(chain) = smartmodule_chain_from_config(config).await? {
        Ok((fluvio, producer.with_chain(chain)?))
    } else {
        Ok((fluvio, producer))
    }
}
