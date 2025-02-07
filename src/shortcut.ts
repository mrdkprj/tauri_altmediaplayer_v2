export const handleShortcut = (renderer: RendererName, e: KeyboardEvent): Mp.ContextMenuEvent | null => {
    if (renderer === "Player") {
        return handlePlayerShortcut(e);
    }

    if (renderer === "Playlist") {
        return handlePlaylistShortcut(e);
    }

    return null;
};

const handlePlayerShortcut = (e: KeyboardEvent): Mp.ContextMenuEvent | null => {
    if (e.key === "F11") {
        e.preventDefault();
        return { id: "ToggleFullscreen", name: "" };
    }

    if (e.ctrlKey && e.key === "s") {
        e.preventDefault();
        return { id: "Capture", name: "" };
    }

    if (e.ctrlKey && e.shiftKey && e.key === "P") {
        e.preventDefault();
        return { id: "ViewSettingsJson", name: "" };
    }

    if (e.ctrlKey && e.key === "p") {
        e.preventDefault();
        return { id: "TogglePlaylistWindow", name: "" };
    }

    return null;
};

const handlePlaylistShortcut = (e: KeyboardEvent): Mp.ContextMenuEvent | null => {
    console.log(e.key);
    if (e.ctrlKey && e.key === "p") {
        e.preventDefault();
        return null;
    }

    if (e.key === "Delete") {
        e.preventDefault();
        return { id: "Remove", name: "" };
    }

    if (e.shiftKey && e.key === "Delete") {
        e.preventDefault();
        return { id: "Trash", name: "" };
    }

    if (e.ctrlKey && e.shiftKey && e.key === "C") {
        e.preventDefault();
        return { id: "CopyFullpath", name: "" };
    }

    if (e.ctrlKey && e.key === "c") {
        e.preventDefault();
        return { id: "CopyFileName", name: "" };
    }

    if (e.ctrlKey && e.key === "v") {
        return { id: "PasteFilePath", name: "" };
    }

    if (e.ctrlKey && e.key === "r") {
        e.preventDefault();
        return { id: "Reveal", name: "" };
    }

    if (e.key == "F2") {
        e.preventDefault();
        return { id: "Rename", name: "" };
    }

    return null;
};
