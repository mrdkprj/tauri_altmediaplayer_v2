export const APP_NAME = "AltMediaPlayer";
export const SEPARATOR = navigator.userAgent.includes("Windows") ? "\\" : "/";
export const handleKeyEvent = () => {
    /**/
};

export const EmptyFile: Mp.MediaFile = {
    id: "",
    fullPath: "",
    src: "",
    name: "",
    date: 0,
    extension: "",
    dir: "",
};

export const PlayableVideoFormats = ["mp4", "mov", "webm"];

export const PlayableVideoExtentions = [".mp4", ".mov", ".webm"];

export const PlayableAudioFormats = ["mp3", "webm"];

export const PlayableAudioExtentions = [".mp3", ".webm"];

export const Resolutions = {
    SizeNone: "",
    "360p": "480x360",
    "480p": "640x480",
    "720p": "1280x720",
    "1080p": "1920x1080",
};

export const Rotations = {
    RotationNone: 0,
    "90Clockwise": 1,
    "90CounterClockwise": 2,
};

export const FORWARD = 1;
export const BACKWARD = -1;
export const Buttons = {
    left: 0,
    right: 2,
};

export const AudioExtensions = [
    ".302",
    ".aac",
    ".ac3",
    ".adts",
    ".adx",
    ".afc",
    ".aif",
    ".aifc",
    ".aiff",
    ".al",
    ".amr",
    ".apm",
    ".aptx",
    ".aptxhd",
    ".ast",
    ".au",
    ".aud",
    ".bit",
    ".c2",
    ".caf",
    ".cvg",
    ".dfpwm",
    ".dts",
    ".eac3",
    ".ec3",
    ".flac",
    ".g722",
    ".gsm",
    ".ircam",
    ".latm",
    ".lbc",
    ".loas",
    ".m2a",
    ".mlp",
    ".mmf",
    ".mp2",
    ".mp3",
    ".mpa",
    ".msbc",
    ".oga",
    ".oma",
    ".opus",
    ".pcm",
    ".rco",
    ".rso",
    ".sb",
    ".sbc",
    ".sf",
    ".sox",
    ".spdif",
    ".spx",
    ".sw",
    ".tco",
    ".thd",
    ".tta",
    ".tun",
    ".ub",
    ".ul",
    ".uw",
    ".vag",
    ".voc",
    ".w64",
    ".wav",
    ".wv",
];

export const VideoExtensions = [
    ".264",
    ".265",
    ".3g2",
    ".3gp",
    ".A64",
    ".a64",
    ".amv",
    ".apng",
    ".asf",
    ".avi",
    ".avif",
    ".avs",
    ".avs2",
    ".avs3",
    ".bmp",
    ".cavs",
    ".chk",
    ".cpk",
    ".dnxhd",
    ".dnxhr",
    ".dpx",
    ".drc",
    ".dv",
    ".dvd",
    ".exr",
    ".f4v",
    ".fits",
    ".flm",
    ".flv",
    ".gif",
    ".gxf",
    ".h261",
    ".h263",
    ".h264",
    ".h265",
    ".hdr",
    ".hevc",
    ".ico",
    ".im1",
    ".im24",
    ".im8",
    ".isma",
    ".ismv",
    ".ivf",
    ".j2c",
    ".j2k",
    ".jls",
    ".jp2",
    ".jpeg",
    ".jpg",
    ".jxl",
    ".ljpg",
    ".m1v",
    ".m2t",
    ".m2ts",
    ".m2v",
    ".m3u8",
    ".m4a",
    ".m4b",
    ".m4v",
    ".mjpeg",
    ".mjpg",
    ".mkv",
    ".mov",
    ".mp4",
    ".mpd",
    ".mpeg",
    ".mpg",
    ".mts",
    ".mxf",
    ".nut",
    ".obu",
    ".ogg",
    ".ogv",
    ".pam",
    ".pbm",
    ".pcx",
    ".pfm",
    ".pgm",
    ".pgmyuv",
    ".phm",
    ".pix",
    ".png",
    ".ppm",
    ".psp",
    ".qoi",
    ".ra",
    ".ras",
    ".rcv",
    ".rgb",
    ".rm",
    ".roq",
    ".rs",
    ".sgi",
    ".sun",
    ".sunras",
    ".swf",
    ".tga",
    ".tif",
    ".tiff",
    ".ts",
    ".vbn",
    ".vc1",
    ".vc2",
    ".vob",
    ".wbmp",
    ".webm",
    ".webp",
    ".wma",
    ".wmv",
    ".wtv",
    ".xbm",
    ".xface",
    ".xwd",
    ".y",
    ".y4m",
    ".yuv",
];
