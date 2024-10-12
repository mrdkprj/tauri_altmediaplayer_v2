import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { join } from "@tauri-apps/api/path";
import { exists, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

const defaultSettings: Mp.Settings = {
    bounds: { width: 1200, height: 800, x: 0, y: 0 },
    playlistBounds: { width: 400, height: 700, x: 0, y: 0 },
    isMaximized: false,
    playlistVisible: true,
    theme: "dark",
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

    constructor() {
        this.data = defaultSettings;
    }

    async init(dataDir: string): Promise<Mp.Settings> {
        const settingPath = await join(dataDir, "temp", "altmediaplayer.settings.json");
        const fileExists = await exists(settingPath);

        if (fileExists) {
            const rawData = await readTextFile(settingPath);
            this.data = this.createSettings(JSON.parse(rawData));
        } else {
            await writeTextFile(settingPath, JSON.stringify(this.data));
        }

        return this.data;
    }

    async save(dataDir: string) {
        const settingPath = await join(dataDir, "temp", "altmediaplayer.settings.json");
        console.log(this.data);
        await writeTextFile(settingPath, JSON.stringify(this.data));
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
