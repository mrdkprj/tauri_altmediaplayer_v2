import { appDataDir } from "@tauri-apps/api/path";
import path from "./path";
import util from "./util";
import { IPCBase } from "./ipc";

const ipc = new IPCBase();
const JSON_NAME = "taltmediaplayer.settings.json";

const defaultSettings: Mp.Settings = {
    bounds: { width: 1200, height: 800, x: 0, y: 0 },
    playlistBounds: { width: 400, height: 700, x: 0, y: 0 },
    isMaximized: false,
    playlistVisible: true,
    theme: "Dark",
    sort: {
        order: "NameAsc",
        groupBy: false,
    },
    video: {
        playbackSpeed: 1,
        seekSpeed: 10,
        fitToWindow: true,
    },
    audio: {
        volume: 1,
        ampLevel: 0.07,
        mute: false,
    },
    defaultPath: "",
    locale: {
        mode: "system",
        lang: "en",
    },
    tags: [],
};

export class Settings {
    data: Mp.Settings;
    private dataDir = "";
    private file = "";

    constructor() {
        this.data = defaultSettings;
    }

    async init(): Promise<Mp.Settings> {
        this.dataDir = await appDataDir();
        const settingPath = path.join(this.dataDir, "temp");
        this.file = path.join(settingPath, JSON_NAME);
        const fileExists = await util.exists(this.file);

        if (fileExists) {
            const rawData = await ipc.invoke("read_text_file", this.file);
            if (rawData) {
                this.data = this.createSettings(JSON.parse(rawData));
            }
        } else {
            await ipc.invoke("mkdir_all", settingPath);
            await ipc.invoke("create", this.file);
            await ipc.invoke("write_text_file", { fullPath: settingPath, data: JSON.stringify(this.data) });
        }

        return this.data;
    }

    async save() {
        await ipc.invoke("write_text_file", { fullPath: this.file, data: JSON.stringify(this.data) });
    }

    getSettingsFilePath() {
        return this.file;
    }

    // private setLanguage(langs:string[]){

    //     if(this.data.locale.mode == "system"){

    //         if(langs[0].includes("ja")){
    //             this.data.locale.lang = "ja"
    //         }else{
    //             this.data.locale.lang = "en"
    //         }

    //     }else{

    //         this.data.locale.lang = this.data.locale.mode
    //     }

    // }

    private createSettings(rawSettings: any): Mp.Settings {
        const config = { ...defaultSettings } as any;

        Object.keys(rawSettings).forEach((key) => {
            if (!(key in config)) return;

            const value = rawSettings[key];

            if (typeof value === "object" && !Array.isArray(value)) {
                Object.keys(value).forEach((valueKey) => {
                    if (valueKey in config[key]) {
                        config[key][valueKey] = value[valueKey];
                    }
                });
            } else {
                config[key] = value;
            }
        });

        return config;
    }
}
