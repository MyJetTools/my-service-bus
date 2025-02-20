var HtmlSessions = /** @class */ (function () {
    function HtmlSessions() {
    }
    HtmlSessions.renderSessions = function (status) {
        var result = '<table style="font-size:12px" class="table table-striped table-dark">' +
            '<tr><th style="width:50px">Id</th><th style="width:120px">Info</th><th>Publisher</th><th>Subscriber</th></tr>';
        for (var _i = 0, _a = status.sessions.items.sort(function (a, b) { return a.id > b.id ? 1 : -1; }); _i < _a.length; _i++) {
            var session = _a[_i];
            var tp = "";
            if (session.type.indexOf("tcp") >= 0) {
                tp = "<span class=\"badge badge-success\">tcp</span>";
            }
            else {
                tp = "<span class=\"badge badge-warning\">" + session.type + "</span>";
            }
            var env_info = "";
            if (session.envInfo) {
                env_info = "<div><span class=\"badge badge-light\">".concat(session.envInfo, "</span></div>");
            }
            result += '<tr class="filter-line"><td>' + session.id + '<div>' + tp + '</div></td>' +
                '<td><b>' + session.name + '</b><div>' + session.version + '</div>' +
                '<div>' + env_info + ' </div>' +
                '<div><b>Ip:</b>' + session.ip + '</div>' +
                '<div id="session-info-' + session.id + '">' + this.renderSessionData(session) + '</div>' +
                '</td>' +
                '<td id="session-topics-' + session.id + '">' + this.renderSessionTopics(status, session) + '</td>' +
                '<td id="session-queues-' + session.id + '">' + this.renderSessionQueues(status, session) + '</td></tr>';
        }
        return result + "</table>";
    };
    HtmlSessions.renderSessionData = function (session) {
        return '<div><b>Connected:</b>' + session.connected + '</div>' +
            '<div><b>Last incoming:</b>' + session.lastIncoming + '</div>' +
            '<div><b>Read:</b>' + Utils.format_bytes(session.readSize) + '</div>' +
            '<div><b>Written:</b>' + Utils.format_bytes(session.writtenSize) + '</div>' +
            '<div><b>Read/sec:</b>' + Utils.format_bytes(session.readPerSec) + '</div>' +
            '<div><b>Written/sec:</b>' + Utils.format_bytes(session.writtenPerSec) + '</div>';
    };
    HtmlSessions.renderSessionQueues = function (status, session) {
        var result = "";
        Iterators.queueSubscribersBySession(status, session.id, function (topic, subscriber) {
            var badgeType = subscriber.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">[' + subscriber.id + ']' + topic.id + " -> " + subscriber.queueId + '</span> ';
        });
        return result;
    };
    HtmlSessions.renderSessionTopics = function (status, session) {
        var result = "";
        Iterators.topicPublishersBySession(status, session.id, function (topic, publisher) {
            var badgeType = publisher.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">' + topic.id + '</span> ';
        });
        return result;
    };
    HtmlSessions.updateSessionData = function (status) {
        for (var _i = 0, _a = status.sessions.items; _i < _a.length; _i++) {
            var session = _a[_i];
            var el = document.getElementById('session-info-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionData(session);
            }
            var el = document.getElementById('session-topics-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionTopics(status, session);
            }
            var el = document.getElementById('session-queues-' + session.id);
            if (el) {
                el.innerHTML = this.renderSessionQueues(status, session);
            }
        }
    };
    return HtmlSessions;
}());
//# sourceMappingURL=HtmlSessions.js.map