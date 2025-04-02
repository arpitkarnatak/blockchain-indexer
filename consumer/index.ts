import { MongoClient, Db } from "mongodb";
import { connect, Connection, Channel } from "amqplib";
import { messageTransform, IndexerMessage } from "./messageTransform";

interface MongoDbInstance {
  client: MongoClient;
  db: Db;
}

interface RabbitMqInstance {
  channel: Channel;
  consumer: (callback: (message: any) => Promise<void>) => void;
}

async function createMongoDbSession(): Promise<MongoDbInstance> {
  const uri = "mongodb://root:example@localhost:27017/";
  const client = new MongoClient(uri);
  await client.connect();
  console.log("Connected to MongoDB");
  const db = client.db("mydatabase");
  console.log("Connected to Database");
  return { client, db };
}

async function createRabbitMqInstance(): Promise<RabbitMqInstance> {
  const connection = await connect(
    "amqp://rabbitmq:rabbitmq@localhost:5672/%2F"
  );
  const channel = await connection.createChannel();
  const queue = "eth_events";
  await channel.assertQueue(queue, { durable: true });
  console.log("Connected to RabbitMQ");

  return {
    channel,
    consumer: async (callback) => {
      await channel.consume(queue, async (msg) => {
        if (msg !== null) {
          const messageContent = JSON.parse(msg.content.toString());
          await callback(messageContent);
          channel.ack(msg);
        }
      });
    },
  };
}

async function main() {
  const rabbitMqInstance = await createRabbitMqInstance();
  const mongoDbInstance = await createMongoDbSession();
  const collection = mongoDbInstance.db.collection("events");

  rabbitMqInstance.consumer(async (message) => {
    const databaseRow = await messageTransform(message);
    try {
      await collection.insertOne({ ...databaseRow, timestamp: new Date().toUTCString() });
      console.log("Inserted message into MongoDB:", databaseRow);
    } catch (error) {
      console.error("Failed to insert message into MongoDB:", error);
    }
  });
}

main().catch(console.error);
