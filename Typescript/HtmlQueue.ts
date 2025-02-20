class HtmlQueue {


    static renderQueueSubscribersCountBadge(count: number): string {

        let badgeClass = count > 0 ? "primary" : "danger";
        return '<span class="badge badge-' + badgeClass + '">' + count.toString() + '<div style="width: 10px; height:10px;display: inline-block;margin-left: 3px;">' + PlugIcon.getIcon() + "</div></span>";
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

    static getQueueSizeBadgeType(queue: ITopicQueue): string {
        if (queue.size > 100) {
            return "badge-danger";
        }

        if (queue.onDelivery > 0) {
            return "badge-warning";
        }

        return "badge-success";
    }

    static renderQueueSizeBadge(queue: ITopicQueue): string {
        let badgeType = this.getQueueSizeBadgeType(queue);
        return '<span class="badge ' + badgeType + '">Size:' + queue.size + "/" + queue.onDelivery + "</span>";
    }


    static renderQueueRanges(queue: ITopicQueue): string {
        let content = "";
        let badgeType = queue.data.length == 1 ? "badge-success" : "badge-danger";

        for (let itm of queue.data) {
            content += '<span class="badge ' + badgeType + '">' + Utils.highlightPageOfMessageId(itm.fromId.toString()) + "-" + Utils.highlightPageOfMessageId(itm.toId.toString()) + "</span> ";
        }

        return content;
    }


    public static renderQueueSubscribers(subscribers: IQueueSubscriber[]): string {

        let html = "";


        for (var itm of subscribers) {

            let subscriber_badge = "badge-primary";


            if (itm.subscriber.deliveryState == 1) {
                subscriber_badge = "badge-warning";
            }
            else
                if (itm.subscriber.deliveryState == 2) {
                    subscriber_badge = "badge-danger";
                }

            let env_info = "";
            if (itm.session.envInfo) {
                env_info = itm.session.envInfo;
            }

            html += `<table class="table-dark" style="width:200px; box-shadow: 0 0 3px black;"">
<tr>
<td>${HtmlMain.drawLed(itm.subscriber.active > 0, 'blue')}<div style="margin-top: 10px;font-size: 12px;"><span class="badge badge-secondary">${itm.session.id}</span></div>
<div style="margin-top: 10px;font-size: 12px;"><span class="badge ${subscriber_badge}">${itm.subscriber.id}</span></div>
</td>
<td padding: 0;">
<div style="text-align:right;padding: 0;"><span class="badge ${subscriber_badge}">${itm.subscriber.deliveryStateStr}</span></div>
<div style="font-size:10px; color:white">${itm.session.name}</div><div style="font-size:10px; color:white">${itm.session.version}</div><div style="font-size:10px; color:white">${env_info}</div><div style="font-size:10px; color:white">${itm.session.ip}</div>
${HtmlGraph.renderGraph(itm.subscriber.history, c => Utils.format_duration(c), c => Math.abs(c), c => c < 0)}</td></tr></table>`;
        }

        return html

    }





}