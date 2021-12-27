
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
            if (session.id == sessionId) {
                return session;
            }
        }
    }

    public static topicPublishersBySession(status: IStatusApiContract, sessionId: number, callback: (topic: ITopic, publisher: ITopicPublisherApiContract) => void) {
        for (let topic of status.topics.items) {
            for (let publisher of topic.publishers)
                if (publisher.sessionId = sessionId)
                    callback(topic, publisher);
        }
    }

    public static queueSubscribersBySession(status: IStatusApiContract, sessionId: number, callback: (topic: ITopic, subscriber: ISubscriberApiContract) => void) {
        for (let topic of status.topics.items) {
            for (let subscriber of topic.subscribers)
                if (subscriber.sessionId = sessionId)
                    callback(topic, subscriber);
        }
    }


    public static getQueueSubscribers(status: IStatusApiContract, topicId: string, queueId: string): IQueueSubscriber[] {

        let result = [];

        for (let topic of status.topics.items) {
            for (let subscriber of topic.subscribers) {
                if (topic.id == topicId && subscriber.queueId == queueId) {
                    for (let session of status.sessions.items) {
                        if (session.id == subscriber.sessionId) {
                            result.push({ subscriber, session });
                        }
                    }

                }
            }
        }

        return result;

    }


    public static getTopicPublishers(status: IStatusApiContract, topic: ITopic): ITopicPublisher[] {

        let result: ITopicPublisher[] = [];

        for (let publisher of topic.publishers) {

            let session = this.findSession(status, publisher.sessionId);

            if (session) {
                result.push({ publisher, session });
            }

        }

        return result;
    }
}