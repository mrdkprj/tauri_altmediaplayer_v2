import { convertFileSrc } from "@tauri-apps/api/core";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { Rotations, Resolutions, PLATFROMS } from "./constants";
import { IPCBase } from "./ipc";
import path from "./path";
import { Command } from "./shell";

class Util {
    convertDestFile: string | null;
    child: Command | null;
    ipc = new IPCBase();

    constructor() {
        this.convertDestFile = null;
        this.child = null;
    }

    async exists(path: string) {
        return await this.ipc.invoke("exists", path);
    }

    async toFiles(fullPaths: string[]): Promise<Mp.MediaFile[]> {
        const stats = await this.ipc.invoke("stat_all", fullPaths);
        return stats.map((stat) => {
            const fullPath = stat.full_path;
            const dir = path.dirname(fullPath);
            const name = path.basename(fullPath);

            return {
                id: crypto.randomUUID(),
                fullPath,
                dir,
                src: convertFileSrc(fullPath),
                name: decodeURIComponent(encodeURIComponent(name)),
                date: stat.attribute.mtime_ms ? stat.attribute.mtime_ms : new Date().getTime(),
                extension: path.extname(fullPath),
            };
        });
    }

    async toFile(fullPath: string): Promise<Mp.MediaFile> {
        const statInfo = await this.ipc.invoke("stat", fullPath);
        const dir = path.dirname(fullPath);
        const name = path.basename(fullPath);

        return {
            id: crypto.randomUUID(),
            fullPath,
            dir,
            src: convertFileSrc(fullPath),
            name: decodeURIComponent(encodeURIComponent(name)),
            date: statInfo.mtime_ms ? statInfo.mtime_ms : new Date().getTime(),
            extension: path.extname(fullPath),
        };
    }

    async updateFile(fullPath: string, currentFile: Mp.MediaFile): Promise<Mp.MediaFile> {
        const dir = path.dirname(fullPath);
        const name = path.basename(fullPath);

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
        return a.name.replace(path.extname(a.name), "").localeCompare(b.name.replace(path.extname(a.name), ""));
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

    toPhysicalPosition(bounds: Mp.Bounds) {
        return new PhysicalPosition(bounds.x, bounds.y);
    }

    toPhysicalSize(bounds: Mp.Bounds) {
        return new PhysicalSize(bounds.width, bounds.height);
    }

    toBounds(position: PhysicalPosition, size: PhysicalSize): Mp.Bounds {
        return {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        };
    }

    async showErrorMessage(ex: any) {
        const mgs = ex.message ? ex.message : ex;
        await this.ipc.invoke("message", { dialog_type: "message", message: mgs, kind: "error" });
    }

    async getMediaMetadata(fullPath: string): Promise<Mp.Metadata> {
        if (navigator.userAgent.includes(PLATFROMS.linux)) {
            return await this.getMediaMetadataForLinux(fullPath);
        } else {
            return await this.getMediaMetadataForWindows(fullPath);
        }
    }

    private async getMediaMetadataForWindows(fullPath: string): Promise<Mp.Metadata> {
        try {
            const metadata = (await this.ipc.invoke("get_media_metadata", fullPath)) as Mp.Metadata;
            metadata.Volume = await this.getVolume(fullPath);
            return metadata;
        } catch (_) {
            await this.cleanUp();
            return {} as Mp.Metadata;
        }
    }

    private async getMediaMetadataForLinux(fullPath: string): Promise<Mp.Metadata> {
        const args = ["-hide_banner", "-v", "error", "-print_format", "json", "-show_streams", "-show_format", "-i", fullPath];
        this.child = new Command("binaries/ffprobe", args);
        try {
            const result = await this.child.spawn();
            const metadata = JSON.parse(result.stdout) as Mp.Metadata;
            metadata.Volume = await this.getVolume(fullPath);
            return metadata;
        } catch (_) {
            await this.cleanUp();
            return {} as Mp.Metadata;
        }
    }

    async getVolume(sourcePath: string): Promise<Mp.MediaVolume> {
        const args = ["-i", sourcePath, "-vn", "-af", "volumedetect", "-f", "null", "-"];
        this.child = new Command("binaries/ffmpeg", args);
        try {
            const result = await this.child.spawn();
            await this.finishConvert();
            return this.extractVolumeInfo(result.stderr);
        } catch (ex: any) {
            console.log(ex);
            await this.cleanUp();
            return { n_samples: "N/A", max_volume: "N/A", mean_volume: "N/A" };
        }
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

    async convertAudio(sourcePath: string, destPath: string, options: Mp.ConvertOptions) {
        if (this.child) throw new Error("Process busy");

        const metadata = await this.getMediaMetadata(sourcePath);

        this.convertDestFile = destPath;

        if (!metadata.streams[1].bit_rate) {
            metadata.streams[1].bit_rate = "0";
        }

        const audioBitrate = options.audioBitrate !== "BitrateNone" ? parseInt(options.audioBitrate) : Math.ceil(parseInt(metadata.streams[1].bit_rate) / 1000);
        let audioVolume = options.audioVolume !== "1" ? `volume=${options.audioVolume}dB` : "";

        if (options.maxAudioVolume) {
            const maxVolumeText = metadata.Volume.max_volume;
            const maxVolume = parseFloat(maxVolumeText);
            if (maxVolume >= 0) {
                throw new Error("No max_volume");
            }
            audioVolume = `volume=${maxVolume * -1}dBdb`;
        }

        const args = ["-i", sourcePath, "-y", "-acodec", "libmp3lame", "-b:a", String(audioBitrate)];

        if (audioVolume) {
            args.push("-filter:a");
            args.push(audioVolume);
        }

        args.push("-f");
        args.push("mp3");
        args.push(destPath);

        this.child = new Command("binaries/ffmpeg", args);

        try {
            await this.child.spawn();
            await this.finishConvert();
        } catch (ex: any) {
            await this.cleanUp();
            throw new Error(ex.message ?? ex.status);
        }
    }

    async convertVideo(sourcePath: string, destPath: string, options: Mp.ConvertOptions) {
        if (this.child) throw new Error("Process busy");

        const metadata = await this.getMediaMetadata(sourcePath);

        this.convertDestFile = destPath;

        const size = Resolutions[options.frameSize] ? Resolutions[options.frameSize] : await this.getSize(metadata);
        const rotate = options.rotation != "RotationNone";

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
        if (rotate) {
            args.push(`scale=${size},transpose=${Rotations[options.rotation]}`);
        } else {
            args.push(`scale=${size}`);
        }

        args.push("-f");
        args.push("mp4");
        args.push(destPath);

        this.child = new Command("binaries/ffmpeg", args);
        try {
            await this.child.spawn();
            await this.finishConvert();
        } catch (ex: any) {
            await this.cleanUp();
            throw new Error(ex.message ?? ex.status);
        }
    }

    private async getSize(metadata: Mp.Metadata) {
        const rotation = metadata.streams[0].rotation;

        if (rotation === "-90" || rotation === "90") {
            return `${metadata.streams[0].height}x${metadata.streams[0].width}`;
        }

        return `${metadata.streams[0].width}x${metadata.streams[0].height}`;
    }

    private async finishConvert() {
        await this.cancelConvert();
        this.child = null;
        this.convertDestFile = null;
    }

    async cancelConvert() {
        if (this.child) {
            await this.child.kill();
        }
    }

    private async cleanUp() {
        if (this.convertDestFile) {
            const found = await this.exists(this.convertDestFile);
            if (found) {
                await this.ipc.invoke("remove", this.convertDestFile);
            }
        }

        await this.finishConvert();
    }
}

const util = new Util();

export default util;
