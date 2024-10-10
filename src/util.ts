import { dirname, basename } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { stat } from "@tauri-apps/plugin-fs";
import { message } from "@tauri-apps/plugin-dialog";
import { Child, Command } from "@tauri-apps/plugin-shell";
import { remove, exists } from "@tauri-apps/plugin-fs";
import { Rotations, Resolutions } from "./constants";
import { IPCBase } from "./ipc";

class Util {
    convertDestFile: string | null;
    child: Child | null;
    ipc = new IPCBase();

    constructor() {
        this.convertDestFile = null;
        this.child = null;
    }

    extname = (name: string | undefined) => {
        if (!name) return "";

        if (name.lastIndexOf(".") < 0) return "";

        return name.substring(name.lastIndexOf(".")).toLowerCase();
    };

    async toFile(fullPath: string): Promise<Mp.MediaFile> {
        const statInfo = await stat(fullPath);
        const dir = await dirname(fullPath);
        const name = await basename(fullPath);

        return {
            id: crypto.randomUUID(),
            fullPath,
            dir,
            src: convertFileSrc(fullPath),
            name: decodeURIComponent(encodeURIComponent(name)),
            date: statInfo.mtime ? statInfo.mtime.getTime() : new Date().getTime(),
            extension: this.extname(fullPath),
        };
    }

    async updateFile(fullPath: string, currentFile: Mp.MediaFile): Promise<Mp.MediaFile> {
        const dir = await dirname(fullPath);
        const name = await basename(fullPath);

        return {
            id: currentFile.id,
            fullPath,
            dir,
            src: convertFileSrc(fullPath),
            name: decodeURIComponent(encodeURIComponent(name)),
            date: currentFile.date,
            extension: currentFile.extension,
        };
    }

    shuffle(targets: any[]) {
        const result = [];
        let size = 0;
        let randomIndex = 0;

        while (targets.length > 0) {
            size = targets.length;
            randomIndex = Math.floor(Math.random() * size);

            result.push(targets[randomIndex]);
            targets.splice(randomIndex, 1);
        }

        return result;
    }

    private localCompareName(a: Mp.MediaFile, b: Mp.MediaFile) {
        return a.name.replace(this.extname(a.name), "").localeCompare(b.name.replace(this.extname(a.name), ""));
    }

    sort(files: Mp.MediaFile[], sortOrder: Mp.SortOrder) {
        if (!files.length) return;

        switch (sortOrder) {
            case "NameAsc":
                return files.sort((a, b) => this.localCompareName(a, b));
            case "NameDesc":
                return files.sort((a, b) => this.localCompareName(b, a));
            case "DateAsc":
                return files.sort((a, b) => a.date - b.date || this.localCompareName(a, b));
            case "DateDesc":
                return files.sort((a, b) => b.date - a.date || this.localCompareName(a, b));
        }
    }

    groupBy<T>(items: T[], key: keyof T) {
        return items.reduce<{ [groupKey: string]: T[] }>((acc, current) => {
            (acc[current[key] as unknown as string] = acc[current[key] as unknown as string] || []).push(current);
            return acc;
        }, {});
    }

    sortByGroup(files: Mp.MediaFile[], sortOrder: Mp.SortOrder) {
        if (!files.length) return;

        const groups = this.groupBy(files, "dir");

        const result = Object.values(groups)
            .map((group) => this.sort(group, sortOrder))
            .flat() as Mp.MediaFile[];
        files.length = 0;
        files.push(...result);
    }

    async showErrorMessage(ex: any) {
        const mgs = ex.message ? ex.message : ex;
        await message(mgs, { kind: "error" });
    }

    async getMediaMetadata(fullPath: string): Promise<Mp.Metadata> {
        const args = ["-hide_banner", "-v", "error", "-print_format", "json", "-show_streams", "-show_format", "-i", fullPath];
        const command = Command.sidecar("binaries/ffprobe", args);
        console.log(command);
        return new Promise(async (resolve, reject) => {
            const result: string[] = [];

            command.on("error", async (stderr: any) => {
                console.log(stderr);
                await this.cleanUp();
                reject({});
            });

            command.on("close", async () => {
                console.log(result);
                const metadata = JSON.parse(result.join("\n")) as Mp.Metadata;
                // metadata.Volume = await this.getVolume(fullPath);
                resolve(metadata);
            });
            command.stdout.on("data", (line) => {
                console.log(line);
                result.push(line);
            });
        });
    }

    async getVolume(sourcePath: string): Promise<Mp.MediaVolume> {
        const args = ["-i", sourcePath, "-vn", "-af", "volumedetect", "-f", "null", "-"];

        return new Promise(async (resolve, reject) => {
            const command = Command.sidecar("binaries/ffmpeg", args);

            const result: string[] = [];

            command.on("error", async (stderr: any) => {
                console.log(stderr);
                await this.cleanUp();
                reject({ n_samples: "N/A", max_volume: "N/A", mean_volume: "N/A" });
            });

            command.on("close", () => {
                this.finishConvert();
                resolve(this.extractVolumeInfo(result.join("\n")));
            });

            command.stderr.on("data", (line) => result.push(line));

            this.child = await command.spawn();
        });
    }

    private extractVolumeInfo(std: string): Mp.MediaVolume {
        const n_samples = std.match(/n_samples:\s?([0-9]*)\s?/)?.at(1) ?? "";
        const mean_volume = std.match(/mean_volume:\s?([^ ]*)\s?dB/)?.at(1) ?? "";
        const max_volume = std.match(/max_volume:\s?([^ ]*)\s?dB/)?.at(1) ?? "";
        return {
            n_samples,
            mean_volume,
            max_volume,
        };
    }

    async cancelConvert() {
        if (this.child) {
            await this.child.kill();
        }
    }

    async convertAudio(sourcePath: string, destPath: string, options: Mp.ConvertOptions) {
        if (this.child) throw new Error("Process busy");

        this.convertDestFile = destPath;

        const metadata = await this.getMediaMetadata(sourcePath);

        if (!metadata.streams[1].bit_rate) {
            metadata.streams[1].bit_rate = "0";
        }

        const audioBitrate = options.audioBitrate !== "BitrateNone" ? parseInt(options.audioBitrate) : Math.ceil(parseInt(metadata.streams[1].bit_rate) / 1000);
        let audioVolume = options.audioVolume !== "1" ? `volume=${options.audioVolume}` : "";

        if (options.maxAudioVolume) {
            const maxVolumeText = metadata.Volume.max_volume;
            const maxVolume = parseFloat(maxVolumeText);
            if (maxVolume >= 0) {
                throw new Error("No max_volume");
            }
            audioVolume = `volume=${maxVolume * -1}dB`;
        }

        const args = ["-i", sourcePath, "-y", "-acodec", "libmp3lame", "-b:a", String(audioBitrate)];

        if (audioVolume) {
            args.push("-filter:a");
            args.push(`volume=${audioVolume}dB`);
        }

        args.push("-f");
        args.push("mp3");
        args.push(destPath);

        const command = Command.sidecar("binaries/ffmpeg", args);

        this.child = await command.spawn();

        return new Promise((resolve, reject) => {
            command.on("error", async (err: any) => {
                await this.cleanUp();
                reject(new Error(err.message));
            });

            command.on("close", () => {
                this.finishConvert();
                resolve(undefined);
            });
        });
    }

    async convertVideo(sourcePath: string, destPath: string, options: Mp.ConvertOptions) {
        if (this.child) throw new Error("Process busy");

        this.convertDestFile = destPath;

        const metadata = await this.getMediaMetadata(sourcePath);

        const size = Resolutions[options.frameSize] ? Resolutions[options.frameSize] : await this.getSize(metadata);
        const rotation = Rotations[options.rotation] ? `transpose=${Rotations[options.rotation]}` : "";

        if (!metadata.streams[1].bit_rate) {
            metadata.streams[1].bit_rate = "0";
        }

        const audioBitrate = options.audioBitrate !== "BitrateNone" ? parseInt(options.audioBitrate) : Math.ceil(parseInt(metadata.streams[1].bit_rate) / 1000);

        let audioVolume = options.audioVolume !== "1" ? `volume=${options.audioVolume}` : "";

        if (options.maxAudioVolume) {
            const maxVolumeText = metadata.Volume.max_volume;
            const maxVolume = parseFloat(maxVolumeText);
            if (maxVolume >= 0) {
                throw new Error("No max_volume");
            }
            audioVolume = `volume=${maxVolume * -1}dB`;
        }

        const args = ["-i", sourcePath, "-y", "-acodec", "libmp3lame"];

        if (audioBitrate > 0) {
            args.push("-b:a"), args.push(String(audioBitrate));
        }

        if (audioVolume) {
            args.push("-filter:a");
            args.push(`volume=${audioVolume}dB`);
        }

        args.push("-vcodec");
        args.push("libx264");

        args.push("-filter:v");
        if (rotation) {
            args.push(`scale=${size}`);
        } else {
            args.push(`scale=${size},transpose=${rotation}`);
        }

        args.push("-f");
        args.push("mp4");
        args.push(destPath);

        const command = Command.sidecar("binaries/ffmpeg", args);

        this.child = await command.spawn();

        return new Promise((resolve, reject) => {
            command.on("error", async (err: any) => {
                await this.cleanUp();
                reject(new Error(err.message));
            });

            command.on("close", () => {
                this.finishConvert();
                resolve(undefined);
            });
        });
    }

    private async getSize(metadata: Mp.Metadata) {
        const rotation = metadata.streams[0].rotation;

        if (rotation === "-90" || rotation === "90") {
            return `${metadata.streams[0].height}x${metadata.streams[0].width}`;
        }

        return `${metadata.streams[0].width}x${metadata.streams[0].height}`;
    }

    private finishConvert() {
        this.cancelConvert();
        this.child = null;
        this.convertDestFile = null;
    }

    private async cleanUp() {
        if (this.convertDestFile && (await exists(this.convertDestFile))) {
            await remove(this.convertDestFile);
        }

        this.finishConvert();
    }
}

const util = new Util();

export default util;
