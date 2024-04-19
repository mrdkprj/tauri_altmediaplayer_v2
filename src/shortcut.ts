export const handleShortcut = (renderer:RendererName, e:KeyboardEvent):Mp.ContextMenuEvent | null => {

    if(renderer === "Player"){
        return handlePlayerShortcut(e);
    }

    if(renderer === "Playlist"){
        return handlePlaylistShortcut(e);
    }

    return null;

}

const handlePlayerShortcut = (e:KeyboardEvent):Mp.ContextMenuEvent | null => {

    if(e.key === "F11"){
        e.preventDefault();
        return {label:"Player", id:"ToggleFullscreen", value:""}
    }

    if(e.ctrlKey && e.key === "s"){
        return {label:"Player", id:"Capture", value:""}
    }

    if(e.ctrlKey && e.key === "p"){
        return {label:"Player", id:"TogglePlaylistWindow", value:""}
    }

    return null;

}

const handlePlaylistShortcut = (e:KeyboardEvent):Mp.ContextMenuEvent | null => {


    if(e.key === "Delete"){
        return {label:"Playlist", id:"Remove", value:""}
    }

    if(e.shiftKey && e.key === "Delete"){
        return {label:"Playlist", id:"Trash", value:""}
    }

    if(e.ctrlKey && e.shiftKey && e.key === "C"){
        return {label:"Playlist", id:"CopyFullpath", value:""}
    }

    if(e.ctrlKey && e.key === "c"){
        return {label:"Playlist", id:"CopyFileName", value:""}
    }

    if(e.ctrlKey && e.key === "r"){
        e.preventDefault();
        return {label:"Playlist", id:"Reveal", value:""}
    }

    if(e.key == "F2"){
        return {label:"Playlist", id:"Rename", value:""}
    }

    return null;
}