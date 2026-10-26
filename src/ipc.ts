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

type MessageResult = {
    button: string;
    cancelled: boolean;
};

type TauriCommandMap = {
    mounted: TauriCommand<null, null>;
    prepare_windows: TauriCommand<Mp.TauriSettings, boolean>;
    get_init_args: TauriCommand<null, string[]>;
    open_context_menu: TauriCommand<Mp.Position, null>;
    open_list_context_menu: TauriCommand<Mp.Position, null>;
    open_sort_context_menu: TauriCommand<Mp.Position, null>;
    change_theme: TauriCommand<Mp.Theme, null>;
    reveal: TauriCommand<string, null>;
    trash: TauriCommand<string, null>;
    remove: TauriCommand<string, null>;
    exists: TauriCommand<string, boolean>;
    rename: TauriCommand<RenameInfo, boolean>;
    stat: TauriCommand<string, FileAttribute>;
    mv_all: TauriCommand<MoveInfo, null>;
    is_uris_available: TauriCommand<null, boolean>;
    read_uris: TauriCommand<null, ClipboardData>;
    read_text: TauriCommand<null, string>;
    write_uris: TauriCommand<WriteUriInfo, null>;
    write_text: TauriCommand<string, null>;
    mkdir: TauriCommand<string, null>;
    mkdir_all: TauriCommand<string, null>;
    create: TauriCommand<string, null>;
    read_text_file: TauriCommand<string, string>;
    write_text_file: TauriCommand<WriteFileInfo, null>;
    write_all: TauriCommand<WriteAllFileInfo, null>;
    stat_all: TauriCommand<string[], FileAttributeExt[]>;
    set_play_thumbs: TauriCommand<any, null>;
    set_pause_thumbs: TauriCommand<any, null>;
    spawn: TauriCommand<SpawnOption, CommandResult>;
    kill: TauriCommand<string, null>;
    message: TauriCommand<DialogOptions, MessageResult>;
    save: TauriCommand<FileDialogOptions, FileDialogResult>;
    open: TauriCommand<FileDialogOptions, FileDialogResult>;
    launch: TauriCommand<string, null>;
    listen_file_drop: TauriCommand<string, null>;
    unlisten_file_drop: TauriCommand<null, null>;
    get_media_metadata: TauriCommand<string, any>;
    undo: TauriCommand<null, null>;
    redo: TauriCommand<null, null>;
    register_video: TauriCommand<string, string>;
};

export const toTauriSettings = (settings: Mp.Settings): Mp.TauriSettings => {
    return {
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
