import { writable } from "svelte/store";
import { EmptyFile } from "../constants";

type AppState = {
    loaded:boolean;
    currentFile:Mp.MediaFile;
    isMaximized:boolean;
    isFullScreen:boolean;
    playing:boolean;
    converting:boolean;
    tooltipVisible:boolean;
    autohide:boolean;
    preventAutohide:boolean;
    startFrom:number | undefined;
    media:Mp.MediaState;
}

type AppAction =
| { type: "init"}
| { type: "loaded", value: boolean}
| { type: "currentFile", value: Mp.MediaFile}
| { type: "isMaximized", value: boolean}
| { type: "isFullScreen", value: boolean}
| { type: "playStatus", value: Mp.PlayStatus}
| { type: "converting"}
| { type: "tooltipVisible", value: boolean}
| { type: "currentTime", value: number}
| { type: "mute", value:boolean}
| { type: "fitToWindow", value: boolean}
| { type: "videoDuration", value: number}
| { type: "videoVolume", value: number}
| { type: "ampLevel", value: number}
| { type: "gainNode", value: GainNode}
| { type: "playbackSpeed", value: number}
| { type: "seekSpeed", value: number}
| { type: "startFrom", value:number | undefined}
| { type: "autohide", value:boolean}
| { type: "preventAutohide", value:boolean}

export const initialAppState : AppState = {
    loaded:false,
    currentFile:EmptyFile,
    isMaximized:false,
    isFullScreen:false,
    playing:false,
    converting:false,
    tooltipVisible:false,
    startFrom:0,
    autohide:false,
    preventAutohide:false,
    media:{
        mute:false,
        fitToWindow:false,
        currentTime:0,
        videoDuration:0,
        videoVolume:0,
        ampLevel:0,
        gainNode:null,
        playbackSpeed:0,
        seekSpeed:0
    },
}

const updater = (state: AppState, action: AppAction): AppState => {

    switch (action.type) {

        case "init":
            return {...state,
                playing:false,
                loaded:false,
                currentFile:EmptyFile,
                media:{...state.media,
                    currentTime:0,
                    videoDuration:0,
                },
            };

        case "loaded":
            return {...state, loaded:action.value}

        case "currentFile": {

            if(action.value.src){
                action.value.src = action.value.src + `?${new Date().getTime()}`
                return {...state, currentFile:action.value, loaded:true};
            }

            return {...state, currentFile:action.value, loaded:false};
        }

        case "isMaximized":
            return {...state, isMaximized:action.value};

        case "isFullScreen":
            return {...state, isFullScreen:action.value};

        case "playStatus":{
            const playing = action.value == "playing"
            return {...state, playing};
        }

        case "converting":
            return {...state, converting:!state.converting};

        case "tooltipVisible":
            return {...state, converting:action.value};

        case "startFrom":
            return {...state, startFrom:action.value};

        case "autohide":
            return {...state, autohide:action.value};

        case "preventAutohide":
            return {...state, preventAutohide:action.value};

        case "currentTime":
            return {...state, media:{...state.media, currentTime:action.value}};

        case "mute":
            return {...state, media:{...state.media, mute:action.value}};

        case "fitToWindow":
            return {...state, media:{...state.media, fitToWindow:action.value}};

        case "videoDuration":
            return {...state, media:{...state.media, videoDuration:action.value}};

        case "videoVolume":
            return {...state, media:{...state.media, videoVolume:action.value}};

        case "ampLevel":
            return {...state, media:{...state.media, ampLevel:action.value}};

        case "gainNode":
            return {...state, media:{...state.media, gainNode:action.value}};

        case "playbackSpeed":
            return {...state, media:{...state.media, playbackSpeed:action.value}};

        case "seekSpeed":
            return {...state, media:{...state.media, seekSpeed:action.value}};

        default:
            return state;
    }
};

const store = writable(initialAppState);

export const dispatch = (action:AppAction) => {
    store.update(state => updater(state, action));
}

export const appState = store