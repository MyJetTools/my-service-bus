class HtmlStatusBar {

    private static connected: boolean;
    private static persistQueue: HTMLElement;
    private static connectedEl: HTMLElement;
    private static totalPagesSize: HTMLElement;
    private static msgsPerSec: HTMLElement;
    private static bytesRwPerSec: HTMLElement;
    private static sessions: HTMLElement;
    private static persistenceVersion: HTMLElement;

    private static currentSessionsAmout: number = -1;
    private static currentPersistenceVersion: string = "";


    public static layout(): string {
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
    }

    public static updateSessionsAmount(amount: number) {
        if (!this.sessions) {
            this.sessions = document.getElementById('sessions');
        }

        if (this.currentSessionsAmout != amount) {
            this.sessions.innerHTML = amount.toFixed(0);
            this.currentSessionsAmout = amount;
        }

    }

    public static updateStatusbar(data: IStatusApiContract) {

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
            this.msgsPerSec = document.getElementById('msg-per-sec')
        }

        if (!this.bytesRwPerSec) {
            this.bytesRwPerSec = document.getElementById('bytes-rw-per-sec');
        }

        let sizes = this.getPersistSize(data);

        this.persistQueue.innerHTML = '<span style="color: green">' + sizes.persist_size + '</span>';
        this.totalPagesSize.innerHTML = '<span style="color: green">' + Utils.formatNumber(sizes.pages_size) + '</span>';

        this.msgsPerSec.innerHTML = Utils.formatNumber(sizes.msgs_per_sec);

        this.bytesRwPerSec.innerHTML = Utils.format_bytes(sizes.bytesReadPerSec) + "/" + Utils.format_bytes(sizes.bytesWrittenPerSec)


        document.getElementById('cpu-mem').innerHTML = 'Mem: <span>' + Utils.format_bytes(data.system.usedmem * 1024) + ' of ' + Utils.format_bytes(data.system.totalmem * 1024) + '</span>';

    }


    private static getPersistSize(data: IStatusApiContract): { persist_size: number, pages_size: number, msgs_per_sec: number, bytesReadPerSec: number, bytesWrittenPerSec: number } {
        let persist_size = 0;
        let pages_size = 0;
        let msgs_per_sec = 0;

        let bytesReadPerSec = 0;
        let bytesWrittenPerSec = 0;

        for (let topic of data.topics.items) {
            persist_size += topic.persistSize;

            msgs_per_sec += topic.messagesPerSec;

            for (let page of topic.pages) {
                pages_size += page.size;
            }
        }


        for (let connection of data.sessions.items) {

            bytesReadPerSec += connection.readPerSec;
            bytesWrittenPerSec += connection.writtenPerSec;

        }



        return { persist_size, pages_size, msgs_per_sec, bytesReadPerSec, bytesWrittenPerSec };

    }


    public static updatePersistenceVersion(ver: string) {

        if (!this.persistenceVersion) {
            this.persistenceVersion = document.getElementById('persistence-ver');
        }

        if (this.currentPersistenceVersion != ver) {
            this.persistenceVersion.innerHTML = ver;
            this.currentPersistenceVersion = ver;
        }

    }


    public static updateOffline() {
        if (this.connected) {
            this.connected = false;
            document.getElementById('connected').innerHTML = '<span style="color: red">offline</span>';
        }
    }
}