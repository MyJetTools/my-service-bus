var HtmlTopics = /** @class */ (function () {
    function HtmlTopics() {
    }
    HtmlTopics.updateTopicQueues = function (status) {
        for (var _i = 0, _a = status.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            var html = '<table class="table table-dark" style="width:100%">';
            for (var _b = 0, _c = Iterators.iterateTopicQueues(status, topic); _b < _c.length; _b++) {
                var queue = _c[_b];
                var subscribers = Iterators.getQueueSubscribers(status, topic, queue.id);
                html += '<tr><td style="width:100%"><div' + Utils.copyToClipboardHtml(queue.id) + '>' + queue.id + '</div>' +
                    '<div>' + HtmlQueue.renderQueueSubscribersCountBadge(subscribers.length) + ' ' + HtmlQueue.renderQueueTypeBadge(queue) + " " + HtmlQueue.renderQueueSizeBadge(queue) + " " + HtmlQueue.renderQueueRanges(queue) + '</div></td>' +
                    '<td style="width:100px">' + HtmlQueue.renderQueueSubscribers(subscribers) + '</td>';
            }
            var el = document.getElementById("topic-queues-" + topic.id);
            if (el) {
                el.innerHTML = html + "</table>";
            }
        }
    };
    HtmlTopics.renderTopicData = function (topic) {
        var queuesizeColor = topic.persistSize < 1000 ? "lightgray" : "red";
        var msgPerSecColor = topic.messagesPerSec > 0 ? "white" : "gray";
        var packetsPerSecColor = topic.packetPerSec > 0 ? "white" : "gray";
        var persist = "";
        if (topic.persist) {
            persist = '<div>Persist queue:<span style="color:' + queuesizeColor + '">' + topic.persistSize + '</span></div>';
        }
        else {
            persist = '<div>Persist: <span style="color:blue">disabled</span></div>';
        }
        return '<div>MsgId:' + Utils.highlightPageOfMessageId(topic.messageId.toString()) + '</div>' +
            '<div>Msg/sec: <span style="color:' + msgPerSecColor + '">' + topic.messagesPerSec + '</span></div>' +
            '<div>Req/sec: <span style="color:' + packetsPerSecColor + '">' + topic.packetPerSec + '</span></div>' +
            persist +
            '<div>' + HtmlGraph.renderGraph(topic.publishHistory, function (v) { return v.toString(); }, function (v) { return v; }, function (_) { return false; }) + '</div>' +
            '<div>' + this.renderCachedPages(topic.pages) + '</div>';
    };
    HtmlTopics.renderCachedPages = function (pages) {
        var result = "";
        for (var _i = 0, pages_1 = pages; _i < pages_1.length; _i++) {
            var page = pages_1[_i];
            result +=
                '<div><div>Page:' + page.id + '; Amount:' + page.amount + '; Size: ' + Utils.formatNumber(page.size) + '</div>' +
                    SubPagesWidget.renderPagesWidget(page.subPages) +
                    '</div>';
        }
        return result;
    };
    HtmlTopics.renderTopics = function (topics) {
        var result = '<table class="table table-striped table-dark">' +
            '<tr><th>Topics</th><th>Topic Connections</th><th>Queues</th></tr>';
        for (var _i = 0, _a = topics.items.sort(function (a, b) { return a.id > b.id ? 1 : -1; }); _i < _a.length; _i++) {
            var topic = _a[_i];
            result += '<tr class="filter-line"><td><b' + Utils.copyToClipboardHtml(topic.id) + '>' + topic.id + '</b>' +
                '<div style="font-size:10px" id="topic-data-' + topic.id + '">' + this.renderTopicData(topic) + '</div></td>' +
                '<td id="topic-sessions-' + topic.id + '"></td>' +
                '<td id="topic-queues-' + topic.id + '"></td>';
        }
        return result + "</table>";
    };
    HtmlTopics.updateTopicSessions = function (status) {
        for (var _i = 0, _a = status.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            var html = "";
            for (var _b = 0, _c = Iterators.getTopicPublishers(status, topic).sort(function (a, b) { return a.session.name > b.session.name ? 1 : -1; }); _b < _c.length; _b++) {
                var itm = _c[_b];
                var env_info = "";
                if (itm.session.envInfo) {
                    env_info = itm.session.envInfo;
                }
                html += '<table class="table table-dark" style=" width:100%; box-shadow: 0 0 3px black;"><tr><td>' + HtmlMain.drawLed(itm.publisher.active > 0, 'green') + '<div style="margin-top: 10px;font-size: 12px;"><span class="badge badge-secondary">' + itm.session.id + '</span></div></td>' +
                    '<td><b>' + itm.session.name + '</b><div>' + itm.session.version + '</div><div>' + env_info + '</div><div>' + itm.session.ip + '</div></td></tr></table>';
            }
            var el = document.getElementById("topic-sessions-" + topic.id);
            if (el) {
                el.innerHTML = html;
            }
        }
    };
    HtmlTopics.updateTopicData = function (topics) {
        for (var _i = 0, _a = topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            var el = document.getElementById('topic-data-' + topic.id);
            if (el) {
                el.innerHTML = this.renderTopicData(topic);
            }
        }
    };
    return HtmlTopics;
}());
//# sourceMappingURL=HtmlTopics.js.map