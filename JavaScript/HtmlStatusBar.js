var HtmlStatusBar = /** @class */ (function () {
    function HtmlStatusBar() {
    }
    HtmlStatusBar.layout = function () {
        return '<div id="status-bar">' +
            '<table><tr>' +
            '<td style="padding-left: 5px;min-width: 110px; font-size:12px;">Connected: <b id="connected" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="min-width:100px; font-size:12px;">Sessions: <b id="sessions" style="text-shadow: 0 0 2px white"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="min-width:130px; font-size:12px;">Persist Queue: <b id="persist-queue" style="text-shadow: 0 0 2px white"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="min-width:100px; font-size:12px;">Msgs/sec: <span id="msg-per-sec" style="text-shadow: 0 0 2px white"></span></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="min-width:170px; font-size:12px;">RW/sec: <span id="bytes-rw-per-sec" style="text-shadow: 0 0 2px white"></span></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="padding-left: 5px; min-width:220px; font-size:12px;"><span id="cpu-mem" style="text-shadow: 0 0 2px white;"></span></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="padding-left: 5px; min-width:200px; font-size:12px;">Total pages size:<span id="total-pages-size" style="text-shadow: 0 0 2px white;"></span></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td style="padding-left: 5px; min-width:270px; font-size:12px;">Persistence ver: <span id="persistence-ver" style="text-shadow: 0 0 2px white;"></span></td>' +
            '</tr></table></div>';
    };
    HtmlStatusBar.updateSessionsAmount = function (amount) {
        if (!this.sessions) {
            this.sessions = document.getElementById('sessions');
        }
        if (this.currentSessionsAmount != amount) {
            this.sessions.innerHTML = amount.toFixed(0);
            this.currentSessionsAmount = amount;
        }
    };
    HtmlStatusBar.updateStatusbar = function (data) {
        if (!this.connected) {
            this.connected = true;
            if (!this.connectedEl) {
                this.connectedEl = document.getElementById('connected');
            }
            this.connectedEl.innerHTML = '<span style="color: green">yes</span>';
        }
        if (!this.persistQueue) {
            this.persistQueue = document.getElementById('persist-queue');
        }
        if (!this.totalPagesSize) {
            this.totalPagesSize = document.getElementById('total-pages-size');
        }
        if (!this.msgsPerSec) {
            this.msgsPerSec = document.getElementById('msg-per-sec');
        }
        if (!this.bytesRwPerSec) {
            this.bytesRwPerSec = document.getElementById('bytes-rw-per-sec');
        }
        var sizes = this.getPersistSize(data);
        this.persistQueue.innerHTML = '<span style="color: green">' + sizes.persist_size + '</span>';
        this.totalPagesSize.innerHTML = '<span style="color: green">' + Utils.formatNumber(sizes.pages_size) + '</span>';
        this.msgsPerSec.innerHTML = Utils.formatNumber(sizes.msgs_per_sec);
        this.bytesRwPerSec.innerHTML = Utils.format_bytes(sizes.bytesReadPerSec) + "/" + Utils.format_bytes(sizes.bytesWrittenPerSec);
        document.getElementById('cpu-mem').innerHTML = 'Mem: <span>' + Utils.format_bytes(data.system.usedmem * 1024) + ' of ' + Utils.format_bytes(data.system.totalmem * 1024) + '</span>';
    };
    HtmlStatusBar.getPersistSize = function (data) {
        var persist_size = 0;
        var pages_size = 0;
        var msgs_per_sec = 0;
        var bytesReadPerSec = 0;
        var bytesWrittenPerSec = 0;
        for (var _i = 0, _a = data.topics.items; _i < _a.length; _i++) {
            var topic = _a[_i];
            persist_size += topic.persistSize;
            msgs_per_sec += topic.messagesPerSec;
            for (var _b = 0, _c = topic.pages; _b < _c.length; _b++) {
                var page = _c[_b];
                pages_size += page.size;
            }
        }
        for (var _d = 0, _e = data.sessions.items; _d < _e.length; _d++) {
            var connection = _e[_d];
            bytesReadPerSec += connection.readPerSec;
            bytesWrittenPerSec += connection.writtenPerSec;
        }
        return { persist_size: persist_size, pages_size: pages_size, msgs_per_sec: msgs_per_sec, bytesReadPerSec: bytesReadPerSec, bytesWrittenPerSec: bytesWrittenPerSec };
    };
    HtmlStatusBar.updatePersistenceVersion = function (ver) {
        if (!this.persistenceVersion) {
            this.persistenceVersion = document.getElementById('persistence-ver');
        }
        if (this.currentPersistenceVersion != ver) {
            this.persistenceVersion.innerHTML = ver;
            this.currentPersistenceVersion = ver;
        }
    };
    HtmlStatusBar.updateOffline = function () {
        if (this.connected) {
            this.connected = false;
            document.getElementById('connected').innerHTML = '<span style="color: red">offline</span>';
        }
    };
    HtmlStatusBar.currentSessionsAmount = -1;
    HtmlStatusBar.currentPersistenceVersion = "";
    return HtmlStatusBar;
}());
//# sourceMappingURL=HtmlStatusBar.js.map