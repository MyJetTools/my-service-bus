var HtmlQueue = /** @class */ (function () {
    function HtmlQueue() {
    }
    HtmlQueue.renderQueueSubscribersCountBadge = function (count) {
        var badgeClass = count > 0 ? "primary" : "danger";
        return '<span class="badge badge-' + badgeClass + '">' + count.toString() + '<div style="width: 10px; height:10px;display: inline-block;margin-left: 3px;">' + PlugIcon.getIcon() + "</div></span>";
    };
    HtmlQueue.renderQueueTypeName = function (queue) {
        if (queue.queueType == 0)
            return "permanent";
        if (queue.queueType == 1)
            return "auto-delete";
        if (queue.queueType == 2)
            return "permanent-single-connect";
        return "unknown:" + queue.queueType;
    };
    HtmlQueue.renderQueueTypeBadge = function (queue) {
        var badgeType = queue.queueType == 1 ? "badge-success" : "badge-warning";
        return '<span class="badge ' + badgeType + '">' + this.renderQueueTypeName(queue) + "</span>";
    };
    HtmlQueue.getQueueSizeBadgeType = function (queue) {
        if (queue.size > 100) {
            return "badge-danger";
        }
        if (queue.onDelivery > 0) {
            return "badge-warning";
        }
        return "badge-success";
    };
    HtmlQueue.renderQueueSizeBadge = function (queue) {
        var badgeType = this.getQueueSizeBadgeType(queue);
        return '<span class="badge ' + badgeType + '">Size:' + queue.size + "/" + queue.onDelivery + "</span>";
    };
    HtmlQueue.renderQueueRanges = function (queue) {
        var content = "";
        var badgeType = queue.data.length == 1 ? "badge-success" : "badge-danger";
        for (var _i = 0, _a = queue.data; _i < _a.length; _i++) {
            var itm = _a[_i];
            content += '<span class="badge ' + badgeType + '">' + Utils.highlightPageOfMessageId(itm.fromId.toString()) + "-" + Utils.highlightPageOfMessageId(itm.toId.toString()) + "</span> ";
        }
        return content;
    };
    HtmlQueue.renderQueueSubscribers = function (subscribers) {
        var html = "";
        for (var _i = 0, subscribers_1 = subscribers; _i < subscribers_1.length; _i++) {
            var itm = subscribers_1[_i];
            var subscriber_badge = "badge-primary";
            if (itm.subscriber.deliveryState == 1) {
                subscriber_badge = "badge-warning";
            }
            else if (itm.subscriber.deliveryState == 2) {
                subscriber_badge = "badge-danger";
            }
            html += "<table class=\"table-dark\" style=\"width:200px; box-shadow: 0 0 3px black;\"\">\n                <tr>\n                <td>".concat(HtmlMain.drawLed(itm.subscriber.active > 0, 'blue'), "<div style=\"margin-top: 10px;font-size: 12px;\"><span class=\"badge badge-secondary\">").concat(itm.session.id, "</span></div>\n                <div style=\"margin-top: 10px;font-size: 12px;\"><span class=\"badge ").concat(subscriber_badge, "\">").concat(itm.subscriber.id, "</span></div></td>\n                <td style=\"font-size:10px\">\n                <table><tr><td>\n                <div>").concat(itm.session.name, "</div><div>").concat(itm.session.version, "</div><div>").concat(itm.session.ip, "</div></td>\n                <td><span class=\"badge ").concat(subscriber_badge, "\">").concat(itm.subscriber.deliveryStateStr, "</span></td></tr></table>\n                ").concat(HtmlGraph.renderGraph(itm.subscriber.history, function (c) { return Utils.format_duration(c); }, function (c) { return Math.abs(c); }, function (c) { return c < 0; }), "</td></tr></table>");
        }
        return html;
    };
    return HtmlQueue;
}());
//# sourceMappingURL=HtmlQueue.js.map