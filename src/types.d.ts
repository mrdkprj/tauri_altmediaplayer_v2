import type { FfprobeData } from "fluent-ffmpeg";

declare global {
    type RendererName = "Player" | "Playlist" | "Convert";

    type RendererChannelEventMap = {
        "backend-ready": Mp.Event;
        "all-ready": Mp.Event;
        "toggle-playlist-visible": Mp.Event;
        "contextmenu-event": Mp.ContextMenuEvent;
        "load-playlist": Mp.LoadPlaylistEvent;
        "load-file": Mp.FileLoadEvent;
        "change-playlist": Mp.ChangePlaylistRequest;
        "toggle-play": Mp.Event;
        "toggle-fullscreen": Mp.Event;
        restart: Mp.Event;
        "release-file": Mp.ReleaseFileRequest;
        "file-released": Mp.ReleaseFileResult;
        log: Mp.Logging;
        "toggle-convert": Mp.Event;
        "open-convert": Mp.OpenConvertDialogEvent;
        "settings-updated": Mp.TauriSettings;
    };

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

        type ContextMenuEvent = {
            id: keyof PlayerContextMenuSubTypeMap | keyof PlaylistContextMenuSubTypeMap;
            name: string;
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
        };

        type TauriSettings = {
            data: string;
            theme: Mp.Theme;
            fitToWindow: boolean;
            playbackSpeed: number;
            seekSpeed: number;
            groupBy: boolean;
            order: Mp.SortOrder;
            playerDropTarget: string;
            playlistDropTarget: string;
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

        type MoveUptoSelection = {
            selectId: string | undefined;
            scrollToId: string | undefined;
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

        type LoadPlaylistEvent = {
            files: string[];
        };

        type FileDropEvent = Event & {
            data?: {
                paths: string[];
            };
            paths?: string[];
        };

        type FullscreenChange = {
            fullscreen: boolean;
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

        type OpenConvertDialogEvent = {
            file: MediaFile;
            opener: DialogOpener;
        };

        type ConvertRequest = {
            sourcePath: string;
            convertFormat: ConvertFormat;
            options: ConvertOptions;
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
     * The standard EventTarget.removeEventListener method. Use it to unsubscribe to the message
     * or sharedbufferreceived event.
     * @param type The name of the event to unsubscribe from. Valid values are message and sharedbufferreceived.
     * @param listener The callback to remove from the event.
     * @param options Options to control how the event is handled.
     */
    removeEventListener(type: string, listener: WebViewEventListenerOrEventListenerObject, options?: boolean | EventListenerOptions): void;
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
