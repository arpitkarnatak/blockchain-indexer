# Blockchain Indexer

## Demo


https://github.com/user-attachments/assets/1f390a5a-2d8e-4960-8f3f-6cf47ffc4705



## Plan of action
<img width="1263" alt="Screenshot 2025-04-02 at 11 05 17 PM" src="https://github.com/user-attachments/assets/6c19acc2-920a-4a28-8766-a1cab4533ec9" />
(the blue boxes consist of properties that can be edited to make it ready for your usecase)

1. The indexer listens to events mentioned in the filter.
2. Indexer will be pushing all the events for the filter to the message queue.
3. Message Queue is being used as a mediator between the listener and logger to ensure all events get properly logged. A fire-and-forget style would leave us vulnerable to missing out on events.
4. There's a `transform` function inside the `/consumer` directory. This function will perform some transformation (or keep it as is) before putting the event object on the database.


## Steps to run

1. `docker compose up -d` to start the queue and the database.
2. In one terminal, add the following env variables in an env file in the `indexer` directory.

```
RPC_URL_WS=<WEBSOCKET_RPC_URL>
RPC_URL_HTTP=<HTTP_RPC_URL>
DATABASE_URL=MONGODB_CONNECTION_STRING
```
3. Start the indexer wth the command `cargo run`
4. Head to `/consumer` and start the consumer with `npm run start` (`npm install` first to get the dependencies)

