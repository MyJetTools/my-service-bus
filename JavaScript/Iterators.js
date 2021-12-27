var Iterators = /** @class */ (function () {
    function Iterators() {
    }
    Iterators.findSession = function (status, sessionId) {
        for (var _i = 0, _a = status.sessions.items; _i < _a.length; _i++) {
            var session = _a[_i];
            if (session.id == sessionId) {
                return session;
            }
        }
    };
    Iterators.topicPublishersBySession = function (status, sessionId, callback) {
        for (var _i = 0, _a = status.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            for (var _b = 0, _c = topic.publishers; _b < _c.length; _b++) {
                var publisher = _c[_b];
                if (publisher.sessionId = sessionId)
                    callback(topic, publisher);
            }
        }
    };
    Iterators.queueSubscribersBySession = function (status, sessionId, callback) {
        for (var _i = 0, _a = status.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            for (var _b = 0, _c = topic.subscribers; _b < _c.length; _b++) {
                var subscriber = _c[_b];
                if (subscriber.sessionId = sessionId)
                    callback(topic, subscriber);
            }
        }
    };
    Iterators.getQueueSubscribers = function (status, topicId, queueId) {
        var result = [];
        for (var _i = 0, _a = status.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            for (var _b = 0, _c = topic.subscribers; _b < _c.length; _b++) {
                var subscriber = _c[_b];
                if (topic.id == topicId && subscriber.queueId == queueId) {
                    for (var _d = 0, _e = status.sessions.items; _d < _e.length; _d++) {
                        var session = _e[_d];
                        if (session.id == subscriber.sessionId) {
                            result.push({ subscriber: subscriber, session: session });
                        }
                    }
                }
            }
        }
        return result;
    };
    Iterators.getTopicPublishers = function (status, topic) {
        var result = [];
        for (var _i = 0, _a = topic.publishers; _i < _a.length; _i++) {
            var publisher = _a[_i];
            var session = this.findSession(status, publisher.sessionId);
            if (session) {
                result.push({ publisher: publisher, session: session });
            }
        }
        return result;
    };
    return Iterators;
}());
//# sourceMappingURL=Iterators.js.map