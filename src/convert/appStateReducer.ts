import { writable } from "svelte/store";

type AppState = {
    audioVolume:string;
    maxVolume:boolean;
    converting:boolean;
    convertFormat:Mp.ConvertFormat;
    frameSize:Mp.VideoFrameSize;
    audioBitrate:Mp.AudioBitrate;
    rotation:Mp.VideoRotation;
    sourceFile:string;
    sourceFileFormat:Mp.ConvertFormat;
}

type AppAction =
| { type:"format", value:Mp.ConvertFormat }
| { type: "audioVolume", value:string }
| { type: "maxVolume", value:boolean }
| { type: "converting", value:boolean }
| { type: "convertFormat", value:Mp.ConvertFormat }
| { type: "frameSize", value:Mp.VideoFrameSize }
| { type: "audioBitrate", value:Mp.AudioBitrate }
| { type: "rotation", value:Mp.VideoRotation }
| { type: "sourceFile", value:string }
| { type: "sourceFileFormat", value:Mp.ConvertFormat }

export const initialAppState : AppState = {
    audioVolume:"1",
    maxVolume:false,
    converting:false,
    convertFormat:"MP4",
    frameSize:"SizeNone",
    audioBitrate:"BitrateNone",
    rotation:"RotationNone",
    sourceFile:"",
    sourceFileFormat:"MP4",
}

export const updater = (state: AppState, action: AppAction): AppState => {

    switch (action.type) {

        case "format":
            return {...state, convertFormat:action.value, sourceFileFormat:action.value};

        case "audioVolume":
            return {...state, audioVolume:action.value};

        case "maxVolume":
            return {...state, maxVolume:action.value};

        case "converting":
            return {...state, converting:action.value};

        case "convertFormat":
            return {...state, convertFormat:action.value};

        case "frameSize":
            return {...state, frameSize:action.value};

        case "audioBitrate":
            return {...state, audioBitrate:action.value};

        case "rotation":
            return {...state, rotation:action.value};

        case "sourceFile":
            return {...state, sourceFile:action.value};

        case "sourceFileFormat":
            return {...state, sourceFileFormat:action.value};

        default:
            return state;
    }
};

const store = writable(initialAppState);

export const dispatch = (action:AppAction) => {
    store.update(state => updater(state, action));
}

export const appState = store
