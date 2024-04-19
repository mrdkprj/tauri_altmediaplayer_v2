import { AudioExtensions, VideoExtensions } from "./constants";

const extname = (name:string | undefined) => {

    if(!name) return "";

    if(name.lastIndexOf(".") < 0) return "";

    return name.substring(name.lastIndexOf(".")).toLowerCase()
}

export const getDropFiles = (e:Mp.FileDropEvent) => {

    return e.paths.filter(path => AudioExtensions.includes(extname(path)) || VideoExtensions.includes(extname(path)))

}