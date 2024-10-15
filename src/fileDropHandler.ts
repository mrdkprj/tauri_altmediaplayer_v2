import { AudioExtensions, VideoExtensions } from "./constants";

const extname = (name: string | undefined) => {
    if (!name) return "";

    if (name.lastIndexOf(".") < 0) return "";

    return name.substring(name.lastIndexOf(".")).toLowerCase();
};

export const getDropFiles = (e: Mp.FileDropEvent) => {
    if (!e.data) return [];

    return e.data.filter((file) => AudioExtensions.includes(extname(file.path)) || VideoExtensions.includes(extname(file.path))).map((file) => file.path);
};

export const getTauriDropFiles = (e: Mp.TauriFileDropEvent) => {
    return e.paths.filter((path) => AudioExtensions.includes(extname(path)) || VideoExtensions.includes(extname(path)));
};
