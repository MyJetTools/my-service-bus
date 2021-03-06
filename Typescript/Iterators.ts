
interface IQueueSubscriber {
    subscriber: ISubscriberApiContract,
    session: ISession
}

interface ITopicPublisher {
    publisher: ITopicPublisherApiContract,
    session: ISession
}

class Iterators {

    public static findSession(status: IStatusApiContract, sessionId: number): ISession {
        for (let session of status.sessions.items) {
            if (session.id === sessionId) {
                return session;
            }
        }
    }

    public static topicPublishersBySession(status: IStatusApiContract, sessionId: number, callback: (topic: ITopic, publisher: ITopicPublisherApiContract) => void) {
        for (let topic of status.topics.items)
            for (let publisher of topic.publishers.sort((a, b) => a.sessionId < b.sessionId ? -1 : 1))
                if (publisher.sessionId === sessionId)
                    callback(topic, publisher);

    }

    public static queueSubscribersBySession(status: IStatusApiContract, sessionId: number, callback: (topic: ITopic, subscriber: ISubscriberApiContract) => void) {
        for (let topic of status.topics.items.sort((a, b) => a.id > b.id ? 1 : 0))
            for (let subscriber of topic.subscribers)
                if (subscriber.sessionId === sessionId)
                    callback(topic, subscriber);
    }


    public static getQueueSubscribers(status: IStatusApiContract, topic: ITopic, queueId: string): IQueueSubscriber[] {
        let result: IQueueSubscriber[] = [];

        for (let subscriber of topic.subscribers.sort((a, b) => a.sessionId < b.sessionId ? -1 : 1)) {
            if (subscriber.queueId == queueId) {

                let session = this.findSession(status, subscriber.sessionId);

                if (session) {
                    result.push({ subscriber, session });
                }

            }
        }
        return result;
    }


    public static getTopicPublishers(status: IStatusApiContract, topic: ITopic): ITopicPublisher[] {
        let result: ITopicPublisher[] = [];

        for (let publisher of topic.publishers.sort((a, b) => a.sessionId < b.sessionId ? -1 : 1)) {
            let session = this.findSession(status, publisher.sessionId);
            if (session) {
                result.push({ publisher, session });
            }
        }
        return result;
    }


    public static iterateTopicQueues(status: IStatusApiContract, topic: ITopic): ITopicQueue[] {
        let queues: ITopicQueues = status.queues[topic.id];

        if (!queues)
            return [];

        return queues.queues;

    }
}