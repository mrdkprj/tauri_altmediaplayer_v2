import path from "./path";
import { AudioExtensions, VideoExtensions } from "./constants";

export const getDropFiles = (e: Mp.FileDropEvent) => {
    if (!e.data && !e.paths) return [];

    const paths = e.data ? e.data.paths : e.paths;
    return paths!.filter((fullPath) => AudioExtensions.includes(path.extname(fullPath)) || VideoExtensions.includes(path.extname(fullPath)));
};
