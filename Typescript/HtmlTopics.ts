class HtmlTopics {


    public static updateTopicQueues(status: IStatusApiContract) {

        for (let topic of status.topics.items) {
            let html = '<table class="table table-dark" style="width:100%">';

            for (let queue of Iterators.iterateTopicQueues(status, topic)) {

                let subscribers = Iterators.getQueueSubscribers(status, topic, queue.id);

                html += '<tr><td style="width:100%"><div' + Utils.copyToClipboardHtml(queue.id) + '>' + queue.id + '</div>' +
                    '<div>' + HtmlQueue.renderQueueSubscribersCountBadge(subscribers.length) + ' ' + HtmlQueue.renderQueueTypeBadge(queue) + " " + HtmlQueue.renderQueueSizeBadge(queue) + " " + HtmlQueue.renderQueueRanges(queue) + '</div></td>' +
                    '<td style="width:100px">' + HtmlQueue.renderQueueSubscribers(subscribers) + '</td>';
            }

            let el = document.getElementById("topic-queues-" + topic.id);

            if (el) {
                el.innerHTML = html + "</table>";
            }
        }
    }


    private static renderTopicData(topic: ITopic): string {
        let queuesizeColor = topic.persistSize < 1000 ? "lightgray" : "red";

        let msgPerSecColor = topic.messagesPerSec > 0 ? "white" : "gray";
        let packetsPerSecColor = topic.packetPerSec > 0 ? "white" : "gray";

        let persist = "";
        if (topic.persist) {
            persist = '<div>Persist queue:<span style="color:' + queuesizeColor + '">' + topic.persistSize + '</span></div>'
        } else {
            persist = '<div>Persist: <span style="color:red"><b>disabled</b></span></div>'
        }

        return '<div>MsgId:' + Utils.highlightPageOfMessageId(topic.messageId.toString()) + '</div>' +
            '<div>Msg/sec: <span style="color:' + msgPerSecColor + '">' + topic.messagesPerSec + '</span></div>' +
            '<div>Req/sec: <span style="color:' + packetsPerSecColor + '">' + topic.packetPerSec + '</span></div>' +
            persist +
            '<div>' + HtmlGraph.renderGraph(topic.publishHistory, v => v.toString(), v => v, _ => false) + '</div>' +

            '<div>' + this.renderCachedPages(topic.pages) + '</div>';
    }


    private static renderCachedPages(pages: IPage[]) {
        let result = "";

        for (let page of pages) {
            result +=
                '<div><div>Page:' + page.id + '; Amount:' + page.amount + '; Size: ' + Utils.formatNumber(page.size) + '</div>' +
                SubPagesWidget.renderPagesWidget(page.subPages) +
                '</div>';
        }

        return result;
    }



    public static renderTopics(topics: ITopics): string {

        let result = '<table class="table table-striped table-dark">' +
            '<tr><th>Topics</th><th>Topic Connections</th><th>Queues</th></tr>';


        for (let topic of topics.items.sort((a, b) => a.id > b.id ? 1 : -1)) {
            result += '<tr class="filter-line"><td><b' + Utils.copyToClipboardHtml(topic.id) + '>' + topic.id + '</b>' +
                '<div style="font-size:10px" id="topic-data-' + topic.id + '">' + this.renderTopicData(topic) + '</div></td>' +
                '<td id="topic-sessions-' + topic.id + '"></td>' +
                '<td id="topic-queues-' + topic.id + '"></td>';
        }

        return result + "</table>";
    }





    public static updateTopicSessions(status: IStatusApiContract) {
        for (let topic of status.topics.items) {

            let html = "";

            for (let itm of Iterators.getTopicPublishers(status, topic).sort((a, b) => a.session.name > b.session.name ? 1 : -1)) {

                let env_info = "";

                if (itm.session.envInfo) {
                    env_info = itm.session.envInfo;
                }

                html += '<table class="table table-dark" style=" width:100%; box-shadow: 0 0 3px black;"><tr><td>' + HtmlMain.drawLed(itm.publisher.active > 0, 'green') + '<div style="margin-top: 10px;font-size: 12px;"><span class="badge badge-secondary">' + itm.session.id + '</span></div></td>' +
                    '<td><b>' + itm.session.name + '</b><div>' + itm.session.version + '</div><div>' + env_info + '</div><div>' + itm.session.ip + '</div></td></tr></table>';
            }


            let el = document.getElementById("topic-sessions-" + topic.id);

            if (el) {
                el.innerHTML = html;
            }
        }
    }

    public static updateTopicData(topics: ITopics) {
        for (let topic of topics.items) {
            var el = document.getElementById('topic-data-' + topic.id);

            if (el) {
                el.innerHTML = this.renderTopicData(topic);
            }
        }
    }
}