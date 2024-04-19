const themeMenu = (config:Mp.Settings):Mp.ContextMenuItem[] => {
    const name = "Theme"
    return [
        {
            name,
            value: "Light",
            label:"Light",
            kind:"radio",
            checked: config.theme === "light",
        },
        {
            name,
            value: "Dark",
            label:"Dark",
            kind:"radio",
            checked: config.theme === "dark",
        },
    ]
}

const playbackSpeedMenu = ():Mp.ContextMenuItem[] => {

    const name = "PlaybackSpeed"
    return [
        {
            name,
            label:"0.25",
            kind:"radio",
            value:"0.25"
        },
        {
            name,
            label:"0.5",
            kind:"radio",
            value:"0.5",
        },
        {
            name,
            label:"0.75",
            kind:"radio",
            value:"0.75",
        },
        {
            name,
            label:"1 - Default",
            kind:"radio",
            checked:true,
            value:"1",
        },
        {
            name,
            label:"1.25",
            kind:"radio",
            value:"1.25",
        },
        {
            name,
            label:"1.5",
            kind:"radio",
            value:"1.5",
        },
        {
            name,
            label:"1.75",
            kind:"radio",
            value:"1.75",
        },
        {
            name,
            label:"2",
            kind:"radio",
            value:"2",
        },
    ]
}

const seekSpeedMenu = ():Mp.ContextMenuItem[] => {

    const name = "SeekSpeed"
    return [
        {
            name,
            label:"0.03sec",
            kind:"radio",
            value:"0.03",
        },
        {
            name,
            label:"0.05sec",
            kind:"radio",
            value:"0.05",
        },
        {
            name,
            label:"0.1sec",
            kind:"radio",
            value:"0.1"
        },
        {
            name,
            label:"0.5sec",
            kind:"radio",
            value:"0.5",
        },
        {
            name,
            label:"1sec",
            kind:"radio",
            value:"1"
        },
        {
            name,
            label:"5sec",
            kind:"radio",
            value:"5"
        },
        {
            name,
            label:"10sec - Default",
            kind:"radio",
            checked:true,
            value:"10"
        },
        {
            name,
            label:"20sec",
            kind:"radio",
            value:"20"
        },
    ]

}

export const getPlayerContextMenu = (config:Mp.Settings):Mp.ContextMenuItem[] => {
    return [
        {
            name:"PlaybackSpeed",
            label: "Playback Speed",
            kind:"submenu",
            submenu: playbackSpeedMenu()
        },
        {
            name:"SeekSpeed",
            label: "Seek Speed",
            kind:"submenu",
            submenu: seekSpeedMenu()
        },
        {
            name:"FitToWindow",
            label: "Fit To Window Size",
            kind: "checkbox",
            checked: config.video.fitToWindow,
        },
        { name:"separator", kind: 'separator' },
        {
            name:"TogglePlaylistWindow",
            label: "Playlist",
            kind:"text",
            accelerator: "Ctrl+P",
        },
        {
            name:"ToggleFullscreen",
            label: "Toggle Fullscreen",
            kind:"text",
            accelerator:"F11",
        },
        {
            name:"PictureInPicture",
            label: "Picture In Picture",
            kind:"text",
        },
        { name:"separator", kind: 'separator' },
        {
            name:"Capture",
            label: "Capture",
            kind:"text",
            accelerator: "Ctrl+S",
        },
        { name:"separator", kind: 'separator' },
        {
            name:"Theme",
            label: "Theme",
            kind:"submenu",
            submenu:themeMenu(config)
        },

    ]
}

export const getPlaylistContextMenu = ():Mp.ContextMenuItem[] => {

    return [
        {
            name:"Remove",
            label: "Remove",
            kind:"text",
            accelerator: "Delete",
        },
        {
            name:"Trash",
            label: "Trash",
            kind:"text",
            accelerator: "Shift+Delete",
        },
        { name:"separator", kind:"separator" },
        {
            name:"CopyFileName",
            label: "Copy Name",
            kind:"text",
            accelerator: "Ctrl+C",
        },
        {
            name:"CopyFullpath",
            label: "Copy Full Path",
            kind:"text",
            accelerator: "Ctrl+Shift+C",
        },
        {
            name:"Reveal",
            label: "Reveal in File Explorer",
            kind:"text",
            accelerator: "Ctrl+R",
        },
        { name:"separator", kind:"separator" },
        {
            name:"Rename",
            label: "Rename",
            kind:"text",
            accelerator: "F2",
        },
        {
            name:"Metadata",
            label: "View Metadata",
            kind:"text",
        },
        {
            name:"Convert",
            label: "Convert",
            kind:"text",
        },
        { name:"separator", kind:"separator" },
        {
            name:"LoadList",
            label: "Load Playlist",
            kind:"text",
        },
        {
            name:"SaveList",
            label: "Save Playlist",
            kind:"text",
        },
        { name:"separator", kind:"separator" },
        {
            name:"RemoveAll",
            label: "Clear Playlist",
            kind:"text",
        },
    ]

}

export const getPlaylistSortContextMenu = (config:Mp.Settings):Mp.ContextMenuItem[] => {

    const name = "Sort"
    return [
        {
            name:"GroupBy",
            label: "Group By Directory",
            kind: "checkbox",
            checked: config.sort.groupBy,
        },
        { name:"separator", kind:"separator" },
        {
            name,
            value: "NameAsc",
            label: "Name(Asc)",
            kind: "radio",
            checked: config.sort.order === "NameAsc",
        },
        {
            name,
            value: "NameDesc",
            label: "Name(Desc)",
            kind: "radio",
            checked: config.sort.order === "NameDesc",
        },
        {
            name,
            value: "DateAsc",
            label: "Date(Asc)",
            kind: "radio",
            checked: config.sort.order === "DateAsc",
        },
        {
            name,
            value: "DateDesc",
            label: "Date(Desc)",
            kind: "radio",
            checked: config.sort.order === "DateDesc",
        },
    ]

}
