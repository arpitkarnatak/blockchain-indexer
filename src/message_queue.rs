use core::str;
use futures_util::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, options::*, types::FieldTable,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;

pub struct MessageQueue {
    channel: Channel,
    queue_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TempEventObject {
    from: String,
    to: String,
    value: String,
}

impl MessageQueue {
    // Create a queue object, establish a connection to RabbitMQ
    pub async fn new(queue_name: &str) -> Result<Self, Box<dyn Error>> {
        let connection = Connection::connect(
            "amqp://rabbitmq:rabbitmq@localhost:5672/%2F",
            ConnectionProperties::default(),
        )
        .await
        .map_err(|e| {
            // Optionally log or transform the error
            Box::<dyn Error>::from(format!("Failed to create RabbitMQ connection: {}", e))
        })?;
        let channel = connection.create_channel().await.map_err(|e| {
            // Optionally log or transform the error
            Box::<dyn Error>::from(format!("Failed to create RabbitMQ Channel: {}", e))
        })?;

        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    passive: false,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        Ok(MessageQueue {
            channel,
            queue_name: queue_name.to_string(),
        })
    }

    // Publish a message to the queue
    pub async fn publish_message(&self, message: &str) -> Result<(), Box<dyn Error>> {
        self.channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;

        println!("Message published to {}: {}", self.queue_name, message);
        Ok(())
    }

    pub async fn consume_message(&self) -> Result<(), Box<dyn Error>> {
        let mut message_consumer = self
            .channel
            .basic_consume(
                &self.queue_name,
                "tag-eth-event",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        while let Some(Ok(message)) = &message_consumer.next().await {
            let mut event_object = serde_json::from_str::<TempEventObject>(
                String::from_utf8(message.data.clone())?.as_str(),
            )?;
            println!("Received event object {:?}", &event_object);
            message.ack(BasicAckOptions::default()).await?;
        }
        Ok(())
    }
}
