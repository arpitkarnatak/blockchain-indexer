import * as bigintConversion from 'bigint-conversion'


export type IndexerMessage = {[key: string]: any}

export async function messageTransform(message: IndexerMessage): Promise<IndexerMessage> {
    // Perform the transformation on the event object

    message.event_data = {...message.event_data, value: Number(bigintConversion.hexToBigint(message.event_data.value ?? 0))}
    message.blockNumber = Number(bigintConversion.hexToBigint(message.blockNumber ?? 0))
    message.logIndex = Number(bigintConversion.hexToBigint(message.logIndex ?? 0))
    message.transactionIndex= Number(bigintConversion.hexToBigint(message.transactionIndex ?? 0))

    return message;
}