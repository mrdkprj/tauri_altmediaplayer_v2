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
        return {id:"ToggleFullscreen", value:""}
    }

    if(e.ctrlKey && e.key === "s"){
        e.preventDefault();
        return {id:"Capture", value:""}
    }

    if(e.ctrlKey && e.shiftKey && e.key === "P"){
        e.preventDefault();
        return {id:"ViewSettingsJson", value:""}
    }

    if(e.ctrlKey && e.key === "p"){
        e.preventDefault();
        return {id:"TogglePlaylistWindow", value:""}
    }

    return null;

}

const handlePlaylistShortcut = (e:KeyboardEvent):Mp.ContextMenuEvent | null => {

    if(e.ctrlKey && e.key === "p"){
        e.preventDefault();
        return null;
    }

    if(e.key === "Delete"){
        e.preventDefault();
        return {id:"Remove", value:""}
    }

    if(e.shiftKey && e.key === "Delete"){
        e.preventDefault();
        return {id:"Trash", value:""}
    }

    if(e.ctrlKey && e.shiftKey && e.key === "C"){
        e.preventDefault();
        return {id:"CopyFullpath", value:""}
    }

    if(e.ctrlKey && e.key === "c"){
        e.preventDefault();
        return {id:"CopyFileName", value:""}
    }

    if(e.ctrlKey && e.key === "r"){
        e.preventDefault();
        return {id:"Reveal", value:""}
    }

    if(e.key == "F2"){
        e.preventDefault();
        return {id:"Rename", value:""}
    }

    return null;
}