type AppState = {
    audioVolume: string;
    maxVolume: boolean;
    converting: boolean;
    convertType: Mp.ConvertType;
    videoCodec: string;
    audioCodec: string;
    frameSize: Mp.VideoFrameSize;
    audioBitrate: Mp.AudioBitrate;
    rotation: Mp.VideoRotation;
    sourceFile: string;
    sourceType: Mp.ConvertType;
};

export const appState: AppState = $state({
    audioVolume: "1",
    maxVolume: false,
    converting: false,
    videoCodec: "mp4",
    audioCodec: "mp3",
    convertType: "Video",
    frameSize: "SizeNone",
    audioBitrate: "BitrateNone",
    rotation: "RotationNone",
    sourceFile: "",
    sourceType: "Video",
});
