import path from "./path";
import { AudioExtensions, VideoExtensions } from "./constants";

export const getDropFiles = (e: Mp.FileDropEvent) => {
    if (!e.data) return [];

    return e.data.filter((file) => AudioExtensions.includes(path.extname(file.path)) || VideoExtensions.includes(path.extname(file.path))).map((file) => file.path);
};

export const getTauriDropFiles = (e: Mp.TauriFileDropEvent) => {
    return e.paths.filter((fullPath) => AudioExtensions.includes(path.extname(fullPath)) || VideoExtensions.includes(path.extname(fullPath)));
};
