class HtmlSessions {

    public static renderSessions(status: IStatusApiContract): string {
        let result = '<table style="font-size:12px" class="table table-striped table-dark">' +
            '<tr><th style="width:50px">Id</th><th style="width:120px">Info</th><th>Publisher</th><th>Subscriber</th></tr>';


        for (let session of status.sessions.items.sort((a, b) => a.name > b.name ? 1 : -1)) {
            let tp = "";

            if (session.type == "tcp") {
                tp = `<span class="badge badge-success">tcp</span>`
            } else {
                tp = `<span class="badge badge-warning">` + session.type + `</span>`
            }

            result += '<tr class="filter-line"><td>' + session.id + '<div>' + tp + '</div></td>' +
                '<td><b>' + session.name + '</b><div>' + session.version + '</div>' +
                '<div><b>Ip:</b>' + session.ip + '</div>' +

                '<div id="session-info-' + session.id + '">' + this.renderSessionData(session) + '</div>' +

                '</td>' +

                '<td id="session-topics-' + session.id + '">' + this.renderSessionTopics(status, session) + '</td>' +
                '<td id="session-queues-' + session.id + '">' + this.renderSessionQueues(status, session) + '</td></tr>';
        }

        return result + "</table>";
    }


    private static renderSessionData(session: ISession): string {
        return '<div><b>Connected:</b>' + session.connected + '</div>' +
            '<div><b>Last incoming:</b>' + session.lastIncoming + '</div>' +
            '<div><b>Read:</b>' + Utils.format_bytes(session.readSize) + '</div>' +
            '<div><b>Written:</b>' + Utils.format_bytes(session.writtenSize) + '</div>' +
            '<div><b>Read/sec:</b>' + Utils.format_bytes(session.readPerSec) + '</div>' +
            '<div><b>Written/sec:</b>' + Utils.format_bytes(session.writtenPerSec) + '</div>';
    }



    private static renderSessionQueues(status: IStatusApiContract, session: ISession): string {

        let result = "";


        Iterators.queueSubscribersBySession(status, session.id, (topic, subscriber) => {
            let badgeType = subscriber.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">[' + subscriber.id + ']' + topic.id + " -> " + subscriber.queueId + '</span> ';

        });


        return result;
    }


    private static renderSessionTopics(status: IStatusApiContract, session: ISession): string {

        let result = "";


        Iterators.topicPublishersBySession(status, session.id, (topic, publisher) => {
            let badgeType = publisher.active > 0 ? "badge-success" : "badge-light";
            result += '<span class="badge ' + badgeType + '">' + topic.id + '</span> ';

        });

        return result;
    }

    public static updateSessionData(status: IStatusApiContract) {
        for (let session of status.sessions.items) {
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
    }

}