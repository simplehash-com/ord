use super::*;
use index::event::Event;
use rdkafka::{
  config::FromClientConfigAndContext,
  producer::{BaseProducer, BaseRecord, DeliveryResult, Producer, ProducerContext},
  ClientConfig, ClientContext, Message,
};
use std::env;

#[derive(Debug)]
pub struct CustomContext;

impl ClientContext for CustomContext {}
impl ProducerContext for CustomContext {
  type DeliveryOpaque = ();

  /// This method is called after attempting to send a message to Kafka.
  fn delivery(&self, result: &DeliveryResult, _: Self::DeliveryOpaque) {
    match result {
      Ok(_) => (),
      Err((e, msg)) => {
        log::error!(
          "received error while writing to kafka sink topic {}: {:?}. Error {}",
          msg.topic(),
          msg,
          e
        );
      }
    }
  }
}

pub struct StreamClient {
  producer: BaseProducer<CustomContext>,
  topic: String,
}

impl StreamClient {
  pub fn new() -> Self {
    let mut config = ClientConfig::new();
    config.set(
            "bootstrap.servers",
            env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or("localhost:9092".to_owned()),
          )
          .set(
            "message.timeout.ms",
            env::var("KAFKA_MESSAGE_TIMEOUT_MS").unwrap_or("5000".to_owned()),
          )
          .set(
            "client.id",
            env::var("KAFKA_CLIENT_ID").unwrap_or("ord-producer".to_owned()),
          )
          .set("sasl.mechanisms", "PLAIN")
          .set("security.protocol", "SASL_SSL")
          .set(
            "sasl.username",
            env::var("KAFKA_API_KEY").unwrap_or("".to_owned()),
          )
          .set(
            "sasl.password",
            env::var("KAFKA_API_SECRET").unwrap_or("".to_owned()),
          )
          .set(
            "linger.ms",
            env::var("KAFKA_LINGER_MS").unwrap_or("200".to_owned()),
          )
          .set(
            "compression.codec",
            env::var("KAFKA_COMPRESSION_CODEC").unwrap_or("snappy".to_owned()),
          )
          .set(
            "socket.keepalive.enable",
            env::var("KAFKA_SOCKET_KEEPALIVE_ENABLE").unwrap_or("true".to_owned()),
          )
          .set(
            "metadata.max.age.ms",
            env::var("KAFKA_METADATA_MAX_AGE_MS").unwrap_or("60000".to_owned()),
          )
          // This needs to timeout sooner than the azure LB timeout
          // to avoid unexpected disconnects from the server after long periods of time
          .set(
            "connections.max.idle.ms",
            env::var("KAFKA_CONNECTIONS_MAX_IDLE_MS").unwrap_or("180000".to_owned()),
          );
    let context = CustomContext;
    StreamClient {
      producer: BaseProducer::from_config_and_context(&config, context)
        .expect("failed to create kafka producer"),
      topic: env::var("KAFKA_TOPIC").unwrap_or("ord".to_owned()),
    }
  }

  pub fn emit(&self, event: &Event) -> Result {
    let key = match event {
      Event::InscriptionCreated {
        block_hash,
        block_height,
        charms: _,
        inscription_id,
        location: _,
        parent_inscription_ids: _,
        sequence_number: _,
      } => format!(
        "InscriptionCreated_{}_{}_{}",
        block_height, block_hash, inscription_id
      ),
      Event::InscriptionTransferred {
        block_hash,
        block_height,
        inscription_id,
        new_location: _,
        old_location: _,
        sequence_number: _,
      } => format!(
        "InscriptionTransferred_{}_{}_{}",
        block_height, block_hash, inscription_id
      ),
      Event::RuneBurned {
        block_hash,
        amount: _,
        block_height,
        rune_id,
        txid,
      } => format!(
        "RuneBurned_{}_{}_{}_{}",
        block_height, block_hash, rune_id, txid
      ),
      Event::RuneEtched {
        block_hash,
        block_height,
        rune_id,
        txid,
      } => format!(
        "RuneEtched_{}_{}_{}_{}",
        block_height, block_hash, rune_id, txid
      ),
      Event::RuneMinted {
        block_hash,
        amount: _,
        block_height,
        rune_id,
        txid,
      } => format!(
        "RuneMinted_{}_{}_{}_{}",
        block_height, block_hash, rune_id, txid
      ),
      Event::RuneTransferred {
        block_hash,
        amount: _,
        block_height,
        outpoint: _,
        rune_id,
        txid,
      } => format!(
        "RuneTransferred_{}_{}_{}_{}",
        block_height, block_hash, rune_id, txid
      ),
    };

    let payload = serde_json::to_vec(&event)?;
    self.producer.poll(Duration::from_secs(0));
    loop {
      match self
        .producer
        .send(BaseRecord::to(&self.topic).key(&key).payload(&payload))
      {
        Ok(_) => return Ok(()),
        Err((e, _)) => {
          println!("Error sending message: {:?}. Retrying", e);
          self.producer.poll(Duration::from_secs(0));
          std::thread::sleep(Duration::from_secs(1));
          continue;
        }
      }
    }
  }

  pub fn flush(&self) {
    log::info!("Flushing producer");
    match self.producer.flush(Duration::from_secs(60)) {
      Ok(_) => (),
      Err(e) => {
        log::error!("Error flushing producer: {:?}", e);
        match self.producer.flush(Duration::from_secs(60)) {
          Ok(_) => (),
          Err(e) => log::error!("Error flushing producer: {:?}", e),
        }
      }
    }
  }
}
