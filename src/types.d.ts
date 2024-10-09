declare global {
    interface Window {
        api: Api;
    }

    type RendererName = "Player" | "Playlist" | "Convert" | "Tag";
    type Renderer = { [key in RendererName]: Electron.BrowserWindow | null };

    type MainChannelEventMap = {
        minimize: Mp.Event;
        "sync-settings": Mp.Settings;
        "toggle-maximize": Mp.Event;
        close: Mp.Event;
        shortcut: Mp.ShortcutEvent;
        progress: Mp.ProgressEvent;
        "open-player-context": Mp.Event;
        "play-status-change": Mp.ChangePlayStatusRequest;
        reload: Mp.Event;
        "save-capture": Mp.CaptureEvent;
        "close-playlist": Mp.Event;
        "trash-ready": Mp.TrashRequest;
        "open-playlist-context": Mp.Event;
        "change-playlist-order": Mp.ChangePlaylistOrderRequet;
        "toggle-play": Mp.Event;
        "toggle-shuffle": Mp.Event;
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
        "sync-settings": Mp.Settings;
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
        "clear-playlist": Mp.Event;
        "sort-type-change": Mp.SortType;
        "start-rename": Mp.Event;
        "after-rename": Mp.RenameResult;
        "after-sourcefile-select": Mp.FileSelectResult;
        "open-convert": Mp.OpenConvertDialogEvent;
        "after-convert": Mp.Event;
        "picture-in-picture": Mp.Event;
        "open-tag-editor": Mp.OpenTagEditorEvent;
    };

    interface Api {
        send: <K extends keyof MainChannelEventMap>(channel: K, data: MainChannelEventMap[K]) => void;
        receive: <K extends keyof RendererChannelEventMap>(channel: K, listener: (data: RendererChannelEventMap[K]) => void) => void;
        removeAllListeners: <K extends keyof RendererChannelEventMap>(channel: K) => void;
    }

    namespace Mp {
        type Lang = "en" | "ja";
        type Theme = "dark" | "light";
        type ConvertFormat = "MP4" | "MP3";
        type ThumbButtonType = "Play" | "Pause" | "Previous" | "Next";
        type PlaybackSpeed = 0.25 | 0.5 | 0.75 | 1 | 1.25 | 1.5 | 1.75 | 2;
        type SeekSpeed = 0.03 | 0.05 | 0.1 | 0.5 | 1 | 3 | 5 | 10 | 20;
        type SortOrder = "NameAsc" | "NameDesc" | "DateAsc" | "DateDesc";

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

        type ShortcutEvent = {
            renderer: RendererName;
            menu: keyof PlayerContextMenuSubTypeMap | keyof PlaylistContextMenuSubTypeMap;
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

        type Property = {
            AppUserModelID?: string;
            AppUserModelParentID?: string;
            AppZoneIdentifier?: string;
            AudioChannelCount?: string;
            AudioEncodingBitrate?: string;
            AudioFormat?: string;
            AudioSampleRate?: string;
            AudioSampleSize?: string;
            AudioStreamNumber?: string;
            Author?: string;
            Comment?: string;
            ComputerName?: string;
            ContentType?: string;
            DRMIsProtected?: string;
            DateAccessed?: string;
            DateCreated?: string;
            DateImported?: string;
            DateModified?: string;
            DocumentDateCreated?: string;
            DocumentDateSaved?: string;
            ExpandoProperties?: string;
            FileAttributes?: string;
            FileAttributesDisplay?: string;
            FileExtension?: string;
            FileName?: string;
            FileOwner?: string;
            FilePlaceholderStatus?: string;
            IsFolder?: string;
            IsShared?: string;
            ItemAuthors?: string;
            ItemDate?: string;
            ItemFolderNameDisplay?: string;
            ItemFolderPathDisplay?: string;
            ItemFolderPathDisplayNarrow?: string;
            ItemName?: string;
            ItemNameDisplay?: string;
            ItemNameDisplayWithoutExtension?: string;
            ItemParticipants?: string;
            ItemPathDisplay?: string;
            ItemPathDisplayNarrow?: string;
            ItemType?: string;
            ItemTypeText?: string;
            Kind?: string;
            KindText?: string;
            LastWriterPackageFamilyName?: string;
            LinkTargetExtension?: string;
            LinkTargetParsingPath?: string;
            LinkTargetSFGAOFlags?: string;
            LinkTargetSFGAOFlagsStrings?: string;
            MIMEType?: string;
            MediaDuration?: string;
            MusicAlbumID?: string;
            MusicAlbumTitle?: string;
            MusicDisplayArtist?: string;
            NetworkLocation?: string;
            NotUserContent?: string;
            OfflineAvailability?: string;
            OfflineStatus?: string;
            ParsingName?: string;
            ParsingPath?: string;
            PerceivedType?: string;
            Rating?: string;
            SFGAOFlags?: string;
            SecurityAllowedEnterpriseDataProtectionIdentities?: string;
            SecurityEncryptionOwners?: string;
            SecurityEncryptionOwnersDisplay?: string;
            ShareScope?: string;
            SharedWith?: string;
            SharingStatus?: string;
            ShellSFGAOFlagsStrings?: string;
            Size?: string;
            StorageProviderAggregatedCustomStates?: string;
            SyncTransferStatusFlags?: string;
            ThumbnailCacheId?: string;
            Title?: string;
            VideoCompression?: string;
            VideoEncodingBitrate?: string;
            VideoFourCC?: string;
            VideoFrameHeight?: string;
            VideoFrameRate?: string;
            VideoFrameWidth?: string;
            VideoHorizontalAspectRatio?: string;
            VideoIsSpherical?: string;
            VideoIsStereo?: string;
            VideoOrientation?: string;
            VideoStreamNumber?: string;
            VideoTotalBitrate?: string;
            VideoVerticalAspectRatio?: string;
            VolumeId?: string;
            ZoneIdentifier?: string;
        };

        type Metadata = Property & {
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

        type FileDropEvent = {
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

        type ProgressEvent = {
            progress: number;
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
            currentTime: number;
        };

        type OpenTagEditorEvent = {
            tags: string[];
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

export {};
