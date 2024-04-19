import { writable } from "svelte/store";
import settings from "../settings";

type AppState = {
    menuType:Mp.ContextMenuType;
    items:{[key in Mp.ContextMenuType]:Mp.ContextMenuItem[]};
    menuSizes:{[key in Mp.ContextMenuType]:Mp.Size};
    settingData:Mp.Settings;
    revert:boolean;
}

type AppAction =
| { type: "menu", value:Mp.ContextMenuType}
| { type: "playerMenu", value: Mp.ContextMenuItem[]}
| { type: "playlistMenu", value: Mp.ContextMenuItem[]}
| { type: "setMenuSize", value: {type:Mp.ContextMenuType, size:Mp.Size}}
| { type: "settings"}
| { type: "revert", value:boolean}

export const initialAppState : AppState = {
    menuType:"Player",
    items:{
        "Player":[],
        "Playlist":[],
        "Sort":[]
    },
    menuSizes:{
        "Player":{width:0, height:0},
        "Playlist":{width:0, height:0},
        "Sort":{width:0, height:0},
    },
    settingData:settings.data,
    revert:false,
}

const updater = (state: AppState, action: AppAction): AppState => {

    switch (action.type) {

        case "menu":
            return {...state, menuType:action.value}

        case "playerMenu": {
            let items = {...state.items}
            items["Player"] = action.value;
            return {...state, items}
        }

        case "playlistMenu":{
            let items = {...state.items}
            items["Playlist"] = action.value;
            return {...state, items}
        }

        case "setMenuSize": {
            let menuSizes = {...state.menuSizes}
            menuSizes[action.value.type] = action.value.size;
            return {...state, menuSizes};
        }

        case "settings":
            return {...state, settingData:settings.data};

        case "revert":
            return {...state, revert:action.value};

        default:
            return state;
    }
};

const store = writable(initialAppState);

export const dispatch = (action:AppAction) => {
    store.update(state => updater(state, action));
}

export const appState = store