type AppState = {
    audioVolume: string;
    maxVolume: boolean;
    converting: boolean;
    convertFormat: Mp.ConvertFormat;
    frameSize: Mp.VideoFrameSize;
    audioBitrate: Mp.AudioBitrate;
    rotation: Mp.VideoRotation;
    sourceFile: string;
    sourceFileFormat: Mp.ConvertFormat;
};

export const appState: AppState = {
    audioVolume: "1",
    maxVolume: false,
    converting: false,
    convertFormat: "MP4",
    frameSize: "SizeNone",
    audioBitrate: "BitrateNone",
    rotation: "RotationNone",
    sourceFile: "",
    sourceFileFormat: "MP4",
};
