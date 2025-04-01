use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, options::*, types::FieldTable,
};
use std::{error::Error, process::exit};

pub struct MessageQueue {
    channel: Channel,
    queue_name: String,
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
}
