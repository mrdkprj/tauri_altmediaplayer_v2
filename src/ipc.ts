import { listen, emit, UnlistenFn, once, emitTo, EventName } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type TauriCommand<Req, Res> = {
    Request: Req;
    Response: Res;
};

type RenameInfo = {
    new: string;
    old: string;
};

type MoveInfo = {
    from: string[];
    to: string;
};

type FileAttribute = {
    is_device: boolean;
    is_directory: boolean;
    is_file: boolean;
    is_hidden: boolean;
    is_read_only: boolean;
    is_symbolic_link: boolean;
    is_system: boolean;
    atime_ms: number;
    ctime_ms: number;
    mtime_ms: number;
    birthtime_ms: number;
    size: number;
};

export type FileAttributeExt = {
    full_path: string;
    attribute: FileAttribute;
};

type WriteFileInfo = {
    fullPath: string;
    data: string;
};

type ClipboardOperation = "Copy" | "Move" | "None";
type ClipboardData = {
    operation: ClipboardOperation;
    urls: string[];
};

type WriteUriInfo = {
    fullPaths: string[];
    operation: ClipboardOperation;
};

type WriteAllFileInfo = {
    fullPath: string;
    data: Uint8Array;
};

type SpawnOption = {
    program: string;
    args?: string[];
    cancellation_token: string;
};

export type CommandStatus = {
    success: boolean;
    code?: number;
};

export type CommandResult = {
    status: CommandStatus;
    stdout: string;
    stderr: string;
};

type DialogOptions = {
    dialog_type: "message" | "confirm" | "ask";
    title?: string;
    kind?: "info" | "warning" | "error";
    cancel_id?: number;
    buttons?: string[];
    message: string;
};

type FileFilter = {
    name: string;
    extensions: string[];
};

type OpenProperty = "OpenFile" | "OpenDirectory" | "MultiSelections";

type FileDialogOptions = {
    title?: string;
    default_path?: string;
    filters?: FileFilter[];
    properties?: OpenProperty[];
};

type FileDialogResult = {
    canceled: boolean;
    file_paths: string[];
};

type TauriCommandMap = {
    prepare_windows: TauriCommand<Mp.TauriSettings, boolean>;
    get_init_args: TauriCommand<undefined, string[]>;
    open_context_menu: TauriCommand<Mp.Position, undefined>;
    open_sort_context_menu: TauriCommand<Mp.Position, undefined>;
    change_theme: TauriCommand<Mp.Theme, undefined>;
    set_settings: TauriCommand<Mp.TauriSettings, undefined>;
    reveal: TauriCommand<string, undefined>;
    trash: TauriCommand<string, undefined>;
    remove: TauriCommand<string, undefined>;
    exists: TauriCommand<string, boolean>;
    rename: TauriCommand<RenameInfo, boolean>;
    stat: TauriCommand<string, FileAttribute>;
    mv_all: TauriCommand<MoveInfo, undefined>;
    is_uris_available: TauriCommand<undefined, boolean>;
    read_uris: TauriCommand<undefined, ClipboardData>;
    read_text: TauriCommand<undefined, string>;
    write_uris: TauriCommand<WriteUriInfo, undefined>;
    write_text: TauriCommand<string, undefined>;
    mkdir: TauriCommand<string, undefined>;
    mkdir_all: TauriCommand<string, undefined>;
    create: TauriCommand<string, undefined>;
    read_text_file: TauriCommand<string, string>;
    write_text_file: TauriCommand<WriteFileInfo, undefined>;
    write_all: TauriCommand<WriteAllFileInfo, undefined>;
    stat_all: TauriCommand<string[], FileAttributeExt[]>;
    set_play_thumbs: TauriCommand<any, undefined>;
    set_pause_thumbs: TauriCommand<any, undefined>;
    spawn: TauriCommand<SpawnOption, CommandResult>;
    kill: TauriCommand<string, undefined>;
    message: TauriCommand<DialogOptions, boolean>;
    save: TauriCommand<FileDialogOptions, FileDialogResult>;
    open: TauriCommand<FileDialogOptions, FileDialogResult>;
    launch: TauriCommand<string, undefined>;
    listen_file_drop: TauriCommand<string, undefined>;
    unlisten_file_drop: TauriCommand<undefined, undefined>;
};

export const toTauriSettings = (settings: Mp.Settings): Mp.TauriSettings => {
    return {
        data: JSON.stringify(settings),
        theme: settings.theme,
        fitToWindow: settings.video.fitToWindow,
        playbackSpeed: settings.video.playbackSpeed,
        seekSpeed: settings.video.seekSpeed,
        groupBy: settings.sort.groupBy,
        order: settings.sort.order,
    };
};

export class IPCBase {
    invoke = async <K extends keyof TauriCommandMap>(channel: K, data: TauriCommandMap[K]["Request"]): Promise<TauriCommandMap[K]["Response"]> => {
        return await invoke<TauriCommandMap[K]["Response"]>(channel, {
            payload: data,
        });
    };

    getSettings = async (): Promise<Mp.Settings> => {
        const settingsJSON = await invoke<string>("get_settings");
        return JSON.parse(settingsJSON);
    };

    updateSettings = async (settings: Mp.Settings) => {
        return await invoke("update_settings", { payload: toTauriSettings(settings) });
    };
}

export class IPC extends IPCBase {
    private label: string;
    private funcs: UnlistenFn[] = [];

    constructor(label: RendererName) {
        super();
        this.label = label;
    }

    receiveOnce = async <K extends keyof RendererChannelEventMap>(channel: K, handler: (e: RendererChannelEventMap[K]) => void) => {
        const fn = await once<RendererChannelEventMap[K]>(channel, (e) => handler(e.payload), { target: { kind: "WebviewWindow", label: this.label } });
        this.funcs.push(fn);
    };

    receive = async <K extends keyof RendererChannelEventMap>(channel: K, handler: (e: RendererChannelEventMap[K]) => void) => {
        const fn = await listen<RendererChannelEventMap[K]>(channel, (e) => handler(e.payload), { target: { kind: "WebviewWindow", label: this.label } });
        this.funcs.push(fn);
    };

    receiveTauri = async <T>(event: EventName, handler: (e: T) => void) => {
        const fn = await listen<T>(event, (e) => handler(e.payload), {
            target: { kind: "WebviewWindow", label: this.label },
        });
        this.funcs.push(fn);
    };

    send = async <K extends keyof RendererChannelEventMap>(channel: K, data: RendererChannelEventMap[K]) => {
        await emit(channel, data);
    };

    sendTo = async <K extends keyof RendererChannelEventMap>(rendererName: RendererName, channel: K, data: RendererChannelEventMap[K]) => {
        await emitTo({ kind: "WebviewWindow", label: rendererName }, channel, data);
    };

    release = () => {
        this.funcs.forEach((fn) => fn());
    };
}
