import path from "./path";
import { AudioExtensions, VideoExtensions } from "./constants";

export const getDropFiles = (e: Mp.FileDropEvent) => {
    if (!e.paths) return [];

    return e.paths.filter((fullPath) => AudioExtensions.includes(path.extname(fullPath)) || VideoExtensions.includes(path.extname(fullPath)));
};
