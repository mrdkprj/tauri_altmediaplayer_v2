import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { appDataDir, join } from "@tauri-apps/api/path";
import { exists, readTextFile, writeTextFile, create, mkdir } from "@tauri-apps/plugin-fs";

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

export const toPhysicalPosition = (bounds: Mp.Bounds) => {
    return new PhysicalPosition(bounds.x, bounds.y);
};

export const toPhysicalSize = (bounds: Mp.Bounds) => {
    return new PhysicalSize(bounds.width, bounds.height);
};

export const toBounds = (position: PhysicalPosition, size: PhysicalSize): Mp.Bounds => {
    return {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    };
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
        const settingPath = await join(this.dataDir, "temp");
        this.file = await join(settingPath, JSON_NAME);
        const fileExists = await exists(this.file);

        if (fileExists) {
            const rawData = await readTextFile(this.file);
            if (rawData) {
                this.data = this.createSettings(JSON.parse(rawData));
            }
        } else {
            await mkdir(settingPath, { recursive: true });
            await create(this.file);
            await writeTextFile(settingPath, JSON.stringify(this.data));
        }

        return this.data;
    }

    async save() {
        await writeTextFile(this.file, JSON.stringify(this.data));
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
