use super::*;
use index::event::Event;
use rdkafka::{
  config::FromClientConfig,
  producer::{BaseProducer, BaseRecord},
  ClientConfig,
};
use std::env;

pub struct StreamClient {
  producer: BaseProducer,
  topic: String,
}

impl StreamClient {
  pub fn new() -> Self {
    StreamClient {
      producer: BaseProducer::from_config(
        ClientConfig::new()
          .set(
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
          ),
      )
      .expect("failed to create kafka producer"),
      topic: env::var("KAFKA_TOPIC").unwrap_or("ord".to_owned()),
    }
  }

  pub fn emit(&self, event: &Event) -> Result {
    let key = match event {
      Event::InscriptionCreated {
        block_height,
        charms: _,
        inscription_id,
        location: _,
        parent_inscription_ids: _,
        sequence_number: _,
      } => format!("InscriptionCreated_{}_{}", block_height, inscription_id),
      Event::InscriptionTransferred {
        block_height,
        inscription_id,
        new_location: _,
        old_location: _,
        sequence_number: _,
      } => format!("InscriptionTransferred_{}_{}", block_height, inscription_id),
      Event::RuneBurned {
        amount: _,
        block_height,
        rune_id,
        txid,
      } => format!("RuneBurned_{}_{}_{}", block_height, rune_id, txid),
      Event::RuneEtched {
        block_height,
        rune_id,
        txid,
      } => format!("RuneEtched_{}_{}_{}", block_height, rune_id, txid),
      Event::RuneMinted {
        amount: _,
        block_height,
        rune_id,
        txid,
      } => format!("RuneMinted_{}_{}_{}", block_height, rune_id, txid),
      Event::RuneTransferred {
        amount: _,
        block_height,
        outpoint: _,
        rune_id,
        txid,
      } => format!("RuneTransferred_{}_{}_{}", block_height, rune_id, txid),
    };

    let payload = serde_json::to_vec(&event)?;
    let record = BaseRecord::to(&self.topic).key(&key).payload(&payload);
    match self.producer.send(record) {
      Ok(_) => Ok(()),
      Err((e, _)) => Err(anyhow!("failed to send kafka message: {}", e)),
    }
  }
}
