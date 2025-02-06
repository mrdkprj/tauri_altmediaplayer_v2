import type { FfprobeData } from "fluent-ffmpeg";

declare global {
    type RendererName = "Player" | "Playlist" | "Convert" | "Tag";
    type Renderer = { [key in RendererName]: Electron.BrowserWindow | null };

    type MainChannelEventMap = {
        ready: Mp.ReadyEvent;
        minimize: Mp.Event;
        "toggle-maximize": Mp.Event;
        "toggle-playlist-visible": Mp.Event;
        close: Mp.Event;
        shortcut: Mp.ShortcutEvent;
        "open-player-context": Mp.Event;
        reload: Mp.Event;
        "save-capture": Mp.CaptureEvent;
        "close-playlist": Mp.Event;
        "trash-ready": Mp.TrashRequest;
        "open-playlist-context": Mp.Event;
        "change-playlist-order": Mp.ChangePlaylistOrderRequet;
        "toggle-play": Mp.Event;
        "toggle-fullscreen": Mp.FullscreenChange;
        "close-convert": Mp.Event;
        "request-convert": Mp.ConvertRequest;
        "open-convert-sourcefile-dialog": Mp.OpenFileDialogRequest;
        "request-cancel-convert": Mp.Event;
        "rename-file": Mp.RenameRequest;
        "playlist-item-selection-change": Mp.PlaylistItemSelectionChange;
        "open-sort-context": Mp.Position;
        "media-state-change": Mp.MediaState;
        "close-tag": Mp.Event;
        "save-tags": Mp.SaveTagsEvent;
        error: Mp.ErrorEvent;
    };

    type RendererChannelEventMap = {
        ready: Mp.ReadyEvent;
        "request-settings": Mp.Event;
        "toggle-playlist-visible": Mp.Event;
        "contextmenu-event": Mp.ContextMenuEvent;
        "open-context-menu": Mp.PopupContextMenuRequest;
        "load-playlist": Mp.LoadPlaylistEvent;
        "load-file": Mp.FileLoadEvent;
        "change-playlist": Mp.ChangePlaylistRequest;
        drop: Mp.DropRequest;
        "toggle-play": Mp.Event;
        "toggle-fullscreen": Mp.Event;
        "change-display-mode": Mp.SettingsChangeEvent;
        "capture-media": Mp.Event;
        restart: Mp.Event;
        "release-file": Mp.ReleaseFileRequest;
        "file-released": Mp.ReleaseFileResult;
        log: Mp.Logging;
        "after-toggle-maximize": Mp.ResizeEvent;
        "toggle-convert": Mp.Event;
        "change-playback-speed": Mp.ChangePlaybackSpeedRequest;
        "change-seek-speed": Mp.ChangeSeekSpeedRequest;
        "after-remove-playlist": Mp.RemovePlaylistItemResult;
        "change-sort-order": Mp.SortOrder;
        "toggle-group-by": Mp.Event;
        "clear-playlist": Mp.Event;
        "sort-type-change": Mp.SortType;
        "start-rename": Mp.Event;
        "after-rename": Mp.RenameResult;
        "after-sourcefile-select": Mp.FileSelectResult;
        "open-convert": Mp.OpenConvertDialogEvent;
        "change-default-path": string;
        "after-convert": Mp.Event;
        "picture-in-picture": Mp.Event;
        "open-tag-editor": Mp.Event;
        "change-tags": string[];
        "move-progress": Mp.MoveProgressEvent;
    };

    interface Api {
        send: <K extends keyof MainChannelEventMap>(channel: K, data: MainChannelEventMap[K]) => void;
        receive: <K extends keyof RendererChannelEventMap>(channel: K, listener: (data: RendererChannelEventMap[K]) => void) => void;
        removeAllListeners: <K extends keyof RendererChannelEventMap>(channel: K) => void;
    }

    namespace Mp {
        type Lang = "en" | "ja";
        type Theme = "Dark" | "Light";
        type ConvertFormat = "MP4" | "MP3";
        type ThumbButtonType = "Play" | "Pause" | "Previous" | "Next";
        type PlaybackSpeed = 0.25 | 0.5 | 0.75 | 1 | 1.25 | 1.5 | 1.75 | 2;
        type SeekSpeed = 0.03 | 0.05 | 0.1 | 0.5 | 1 | 3 | 5 | 10 | 20;
        type SortOrder = "NameAsc" | "NameDesc" | "DateAsc" | "DateDesc";
        type ThumbButtonId = "Play" | "Pause" | "Previous" | "Next";

        type PlayerContextMenuSubTypeMap = {
            PlaybackSpeed: Mp.PlaybackSpeed;
            SeekSpeed: Mp.SeekSpeed;
            TogglePlaylistWindow: null;
            FitToWindow: null;
            ToggleFullscreen: null;
            Theme: Mp.Theme;
            Capture: null;
            PictureInPicture: null;
            ViewSettingsJson: null;
        };

        type PlaylistContextMenuSubTypeMap = {
            Remove: null;
            RemoveAll: null;
            Trash: null;
            CopyFileName: null;
            CopyFullpath: null;
            Reveal: null;
            Metadata: null;
            Convert: null;
            Tag: string;
            ManageTags: null;
            Sort: Mp.SortOrder;
            Rename: null;
            Move: null;
            GroupBy: null;
            PasteFilePath: null;
        };

        type VideoFrameSize = "SizeNone" | "360p" | "480p" | "720p" | "1080p";
        type VideoRotation = "RotationNone" | "90Clockwise" | "90CounterClockwise";
        type AudioBitrate = "BitrateNone" | "128" | "160" | "192" | "320";

        type PlayStatus = "playing" | "paused" | "stopped";

        type DialogOpener = "system" | "user";

        type SecondInstanceState = {
            timeout: NodeJS.Timeout | null;
            requireInitPlaylist: boolean;
        };

        type ContextMenuEvent = {
            id: keyof PlayerContextMenuSubTypeMap | keyof PlaylistContextMenuSubTypeMap;
            name: string;
        };

        type PopupContextMenuRequest = {
            type: Mp.ContextMenuType;
            position: Mp.Position;
        };

        type ContextMenuType = "Player" | "Playlist" | "Sort";

        type MenuKind = "text" | "submenu" | "radio" | "checkbox" | "separator";

        type ContextMenuItem = {
            name: keyof PlayerContextMenuSubTypeMap | keyof PlaylistContextMenuSubTypeMap | "separator";
            label?: string;
            value?: string;
            kind: Mp.MenuKind;
            checked?: boolean;
            accelerator?: string;
            submenu?: Mp.ContextMenuItem[];
        };

        type Bounds = {
            width: number;
            height: number;
            x: number;
            y: number;
        };

        type Position = {
            x: number;
            y: number;
        };

        type Size = {
            width: number;
            height: number;
        };

        type SortType = {
            order: SortOrder;
            groupBy: boolean;
        };

        type Settings = {
            bounds: Bounds;
            playlistBounds: Bounds;
            theme: Mp.Theme;
            isMaximized: boolean;
            playlistVisible: boolean;
            sort: Mp.SortType;
            video: {
                fitToWindow: boolean;
                playbackSpeed: number;
                seekSpeed: number;
            };
            audio: {
                volume: number;
                ampLevel: number;
                mute: boolean;
            };
            defaultPath: string;
            locale: {
                mode: "system" | Mp.Lang;
                lang: Mp.Lang;
            };
            tags: string[];
        };

        type MediaFile = {
            id: string;
            fullPath: string;
            dir: string;
            src: string;
            name: string;
            date: number;
            extension: string;
        };

        type MediaState = {
            mute: boolean;
            fitToWindow: boolean;
            currentTime: number;
            videoDuration: number;
            videoVolume: number;
            ampLevel: number;
            gainNode: GainNode | null;
            playbackSpeed: number;
            seekSpeed: number;
        };

        type MediaVolume = {
            n_samples: string;
            mean_volume: string;
            max_volume: string;
        };

        type PlaylistItemSelection = {
            selectedId: string;
            selectedIds: string[];
        };

        type PlaylistDragState = {
            dragging: boolean;
            startElement: HTMLElement | null;
            targetElement: HTMLElement | null;
            startIndex: number;
        };

        type RenameData = {
            id: string;
            name: string;
        };

        type MetadataRequest = {
            fullPath: string;
            format: boolean;
        };

        type Metadata = FfprobeData & {
            Volume: MediaVolume;
        };

        type ConvertOptions = {
            frameSize: VideoFrameSize;
            audioBitrate: AudioBitrate;
            rotation: VideoRotation;
            audioVolume: string;
            maxAudioVolume: boolean;
        };

        type ReadyEvent = {
            settings: Mp.Settings;
        };

        type LoadPlaylistEvent = {
            files: string[];
        };

        type FileDropEvent = Event & {
            data?: Mp.DroppedFile[];
        };

        type DroppedFile = {
            kind: string;
            name: string;
            path: string;
        };

        type TauriFileDropEvent = {
            paths: string[];
            position: Position;
        };

        type ResizeEvent = {
            isMaximized: boolean;
        };

        type FullscreenChange = {
            fullscreen: boolean;
        };

        type DropRequest = {
            files: string[];
        };

        type ChangePlaylistOrderRequet = {
            start: number;
            end: number;
            currentIndex: number;
        };

        type ChangePlaylistRequest = {
            index: number;
        };

        type ChangePlayStatusRequest = {
            status: PlayStatus;
        };

        type FileLoadEvent = {
            currentFile: MediaFile;
            type: "Load" | "Play";
            startFrom?: number;
        };

        type ReplaceFileRequest = {
            file: MediaFile;
        };

        type CaptureEvent = {
            data: string;
            timestamp: number;
        };

        type CloseRequest = {
            mediaState: MediaState;
        };

        type ReleaseFileRequest = {
            fileIds: string[];
        };

        type ReleaseFileResult = {
            playing: boolean;
            currentTime: number;
        };

        type MoveFileRequest = {
            sources: string[];
            dest: string;
            cancellationId: number;
        };

        type MoveProgressEvent = {
            totalFileSize: number;
            transferred: number;
        };

        type SaveTagsEvent = {
            tags: string[];
        };

        type OpenConvertDialogEvent = {
            file: MediaFile;
            opener: DialogOpener;
        };

        type ConvertRequest = {
            sourcePath: string;
            convertFormat: ConvertFormat;
            options: ConvertOptions;
        };

        type FileSelectResult = {
            file: MediaFile;
        };

        type OpenFileDialogRequest = {
            fullPath: string;
        };

        type ErrorEvent = {
            message: string;
        };

        type Event = {
            args?: any;
        };

        type Logging = {
            log: any;
        };

        type RadioGroupChangeEvent<T> = {
            value: T;
        };

        type MessageLabel = {
            selectConvertInputFile: string;
            selectPlaylistFile: string;
            unsupportedMedia: string;
        };

        type Label = {
            restart: string;
            shuffle: string;
            sort: string;
            playbackSpeed: string;
            seekSpeed: string;
            fitToWindow: string;
            playlist: string;
            fullscreen: string;
            pip: string;
            capture: string;
            theme: string;
            light: string;
            dark: string;
            lang: string;
            default: string;
            second: string;
            remove: string;
            trash: string;
            copyName: string;
            copyFullpath: string;
            reveal: string;
            rename: string;
            metadata: string;
            convert: string;
            loadList: string;
            saveList: string;
            clearList: string;
            groupBy: string;
            nameAsc: string;
            nameDesc: string;
            dateAsc: string;
            dateDesc: string;
            play: string;
            pause: string;
            previous: string;
            next: string;
            inputFile: string;
            convertFormat: string;
            frameSize: string;
            videoRotation: string;
            audioBitrate: string;
            volume: string;
            maximizeVolue: string;
            start: string;
            cancel: string;
            close: string;
            mute: string;
            tags: string;
            manageTag: string;
            mediaFile: string;
            playlistFile: string;
        };

        type Labels = Label & MessageLabel;
    }
}

/**
 * window.chrome.webview is the class to access the WebView2-specific APIs that are available
 * to the script running within WebView2 Runtime.
 */
export interface WebView extends EventTarget {
    /**
     * Contains asynchronous proxies for all host objects added via CoreWebView2.AddHostObjectToScript
     * as well as options to configure those proxies, and the container for synchronous proxies.
     *
     * If you call coreWebView2.AddHostObjectToScript("myObject", object); in your native code,
     * an asynchronous proxy for object is available to your web-side code, by using
     * chrome.webview.hostObjects.myObject.
     */
    hostObjects: HostObjectsAsyncRoot;

    /**
     * The standard EventTarget.addEventListener method. Use it to subscribe to the message event
     * or sharedbufferreceived event. The message event receives messages posted from the WebView2
     * host via CoreWebView2.PostWebMessageAsJson or CoreWebView2.PostWebMessageAsString. The
     * sharedbufferreceived event receives shared buffers posted from the WebView2 host via
     * CoreWebView2.PostSharedBufferToScript.
     * See CoreWebView2.PostWebMessageAsJson( Win32/C++, .NET, WinRT).
     * @param type The name of the event to subscribe to. Valid values are message, and sharedbufferreceived.
     * @param listener The callback to invoke when the event is raised.
     * @param options Options to control how the event is handled.
     */
    addEventListener(type: string, listener: WebViewEventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions): void;

    /**
     * When the page calls postMessage, the message parameter is converted to JSON and is posted
     * asynchronously to the WebView2 host process. This will result in either the
     * CoreWebView2.WebMessageReceived event or the CoreWebView2Frame.WebMessageReceived event being
     * raised, depending on if postMessage is called from the top-level document in the WebView2
     * or from a child frame. See CoreWebView2.WebMessageReceived( Win32/C++, .NET, WinRT).
     * See CoreWebView2Frame.WebMessageReceived( Win32/C++, .NET, WinRT).
     * @param message The message to send to the WebView2 host. This can be any object that can be
     *                serialized to JSON.
     */
    postMessage(message: any): void;

    /**
     * Call with the ArrayBuffer from the chrome.webview.sharedbufferreceived event to release the
     * underlying shared memory resource.
     * @param buffer An ArrayBuffer from the chrome.webview.sharedbufferreceived event.
     */
    releaseBuffer(buffer: ArrayBuffer): void;

    /**
     * The standard EventTarget.removeEventListener method. Use it to unsubscribe to the message
     * or sharedbufferreceived event.
     * @param type The name of the event to unsubscribe from. Valid values are message and sharedbufferreceived.
     * @param listener The callback to remove from the event.
     * @param options Options to control how the event is handled.
     */
    removeEventListener(type: string, listener: WebViewEventListenerOrEventListenerObject, options?: boolean | EventListenerOptions): void;

    postMessageWithAdditionalObjects(eventName: string, data: any): void;
}

// Global object
declare global {
    interface Window {
        chrome: {
            webview: WebView;
        };
    }
}
export {};
