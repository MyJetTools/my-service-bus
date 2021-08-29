class HtmlQueue {


    static renderQueueSubscribersCountBadge(count: number): string {

        let badgeClass = count > 0 ? "primary" : "danger";
        return '<span class="badge badge-' + badgeClass + '"><img style="width: 10px" src="/img/plug.svg"> ' + count.toString() + "</span>";
    }


    static renderQueueTypeName(queue: ITopicQueue): string {
        if (queue.queueType == 0)
            return "permanent";

        if (queue.queueType == 1)
            return "auto-delete";

        if (queue.queueType == 2)
            return "permanent-single-connect";

        return "unknown:" + queue.queueType;
    }



    static renderQueueTypeBadge(queue: ITopicQueue): string {

        let badgeType = queue.queueType == 1 ? "badge-success" : "badge-warning";

        return '<span class="badge ' + badgeType + '">' + this.renderQueueTypeName(queue) + "</span>";

    }

    static renderQueueSizeBadge(queue: ITopicQueue): string {

        let badgeType = queue.size > 100 ? "badge-danger" : "badge-success";

        return '<span class="badge ' + badgeType + '">Size:' + queue.size + "</span>";

    }


    static renderQueueRanges(queue: ITopicQueue): string {
        let content = "";
        let badgeType = queue.data.length == 1 ? "badge-success" : "badge-danger";

        for (let itm of queue.data) {
            content += '<span class="badge ' + badgeType + '">' + itm.fromId + "-" + itm.toId + "</span> ";
        }

        return content;
    }


    public static renderQueueSubscribers(subscribers: IQueueSubscriber[]): string {

        let html = "";


        for (var itm of subscribers) {
            html += '<table class="table-dark" style="width:200px; box-shadow: 0 0 3px black;"">' +
                '<tr><td>' + HtmlMain.drawLed(itm.subscriber.active > 0, 'blue') + '<div style="margin-top: 10px;font-size: 12px;"><span class="badge badge-secondary">' + itm.session.id + '</span></div></td>' +
                '<td style="font-size:10px"><div>' + itm.session.name + '</div><div>' + itm.session.version + '</div><div> ' + itm.session.ip + ' </div>' +
                HtmlGraph.renderGraph(itm.subscriber.deliveryHistory, c => Utils.format_duration(c), c => Math.abs(c), c => c < 0) + '</td></tr></table>';

        }

        return html

    }



}