
interface IStatusApiContract {
    topics: ITopics,
    queues: object,
    sessions: ISessions
    system: ISystemStatus,
    persistenceVersion: string
}

interface ISession {
    id: number,
    ip: string,
    type: string,
    version: string,
    name: string,
    connected: string,
    lastIncoming: string,
    readSize: number,
    writtenSize: number,
    readPerSec: number,
    writtenPerSec: number,
}

interface ISessions {
    snapshotId: number,
    items: ISession[]
}

interface ITopics {
    snapshotId: number,
    items: ITopic[],
}

interface ITopicPublisherApiContract {
    sessionId: number,
    active: number
}

interface ITopic {
    id: string,
    messageId: number,
    packetPerSec: number,
    messagesPerSec: number,
    persistSize: number,
    publishHistory: number[],
    pages: IPage[],
    publishers: ITopicPublisherApiContract[],
    subscribers: ISubscriberApiContract[],
    persist: boolean,
}

interface IPage {
    id: number,
    amount: number,
    size: number,
    subPages: number[]
}


interface ITopicQueues {
    snapshotId: number,
    queues: ITopicQueue[]
}

interface ITopicQueue {
    id: string,
    queueType: number,
    size: number,
    onDelivery: number,
    data: IQueueIndexRange[]
}

interface ISubscriberApiContract {
    id: number,
    sessionId: number;
    queueId: string,
    active: number,
    deliveryState: number,
    deliveryStateStr: string,
    history: number[],
}

interface IQueueIndexRange {
    fromId: number,
    toId: number
}

interface ISystemStatus {
    usedmem: number,
    totalmem: number
}

