import { writable } from "svelte/store";

type RenamePartialRect = {
    top: number;
    left: number;
    height: number;
    width: number;
};

type RenameState = {
    renaming: boolean;
    inputValue: string;
    rect: RenamePartialRect;
};

type DragState = {
    dragging: boolean;
    dir: string;
    startId: string;
    targetId: string;
};

type SearchState = {
    searching: boolean;
    itemIds: string[];
    highlighIndex: number;
    value: string;
};

type MoveState = {
    started: boolean;
    progress: number;
};

type AppState = {
    currentIndex: number;
    selection: Mp.PlaylistItemSelection;
    shuffle: boolean;
    sortType: Mp.SortType;
    files: Mp.MediaFile[];
    preventBlur: boolean;
    rename: RenameState;
    dragState: DragState;
    searchState: SearchState;
    moveState: MoveState;
};

export const initialAppState: AppState = {
    currentIndex: -1,
    selection: { selectedId: "", selectedIds: [] },
    shuffle: false,
    sortType: { order: "NameAsc", groupBy: false },
    files: [],
    preventBlur: false,
    rename: {
        renaming: false,
        inputValue: "",
        rect: {
            top: 0,
            left: 0,
            width: 0,
            height: 0,
        },
    },
    dragState: {
        dragging: false,
        dir: "",
        startId: "",
        targetId: "",
    },
    searchState: {
        searching: false,
        itemIds: [],
        highlighIndex: 0,
        value: "",
    },
    moveState: {
        started: false,
        progress: 0,
    },
};

type AppAction =
    | { type: "currentIndex"; value: number }
    | { type: "clear" }
    | { type: "selectedId"; value: string }
    | { type: "setSelectedIds"; value: string[] }
    | { type: "replaceSelectedIds"; value: { id: string; index: number } }
    | { type: "clearSelection" }
    | { type: "updatSelectedIds"; value: string[] }
    | { type: "updateSelection"; value: Mp.PlaylistItemSelection }
    | { type: "sortType"; value: Mp.SortType }
    | { type: "files"; value: Mp.MediaFile[] }
    | { type: "appendFiles"; value: Mp.MediaFile[] }
    | { type: "removeFiles"; value: number[] }
    | { type: "startRename"; value: { rect: RenamePartialRect; value: string } }
    | { type: "endRename" }
    | { type: "preventBlur"; value: boolean }
    | { type: "toggleShuffle" }
    | { type: "rename"; value: Mp.MediaFile }
    | { type: "udpateName"; value: string }
    | { type: "startDrag"; value: { startId: string; dir: string } }
    | { type: "drag"; value: string }
    | { type: "endDrag" }
    | { type: "toggleSearch"; value: boolean }
    | { type: "highlightItems"; value: string[] }
    | { type: "changeHighlight"; value: number }
    | { type: "startMove" }
    | { type: "moveProgress"; value: number }
    | { type: "endMove" };

const updater = (state: AppState, action: AppAction) => {
    switch (action.type) {
        case "currentIndex":
            return { ...state, currentIndex: action.value };

        case "clear":
            return { ...state, currentIndex: -1, files: [], selection: { ...state.selection, selectedId: "", selectedIds: [] } };

        case "selectedId":
            return { ...state, selection: { ...state.selection, selectedId: action.value } };

        case "setSelectedIds":
            return { ...state, selection: { ...state.selection, selectedIds: action.value } };

        case "replaceSelectedIds": {
            const selectedIds = [...state.selection.selectedIds];
            selectedIds[action.value.index] = action.value.id;
            return { ...state, selection: { ...state.selection, selectedIds } };
        }

        case "clearSelection":
            return { ...state, selection: { ...state.selection, selectedId: "", selectedIds: [] } };

        case "updatSelectedIds":
            return { ...state, selection: { ...state.selection, selectedIds: [...state.selection.selectedIds, ...action.value] } };

        case "updateSelection":
            return { ...state, selection: { ...state.selection, selectedId: action.value.selectedId, selectedIds: action.value.selectedIds } };

        case "sortType":
            return { ...state, sortType: action.value };

        case "files":
            return { ...state, files: action.value };

        case "appendFiles": {
            const files = [...state.files];
            files.push(...action.value);
            return { ...state, files };
        }

        case "removeFiles": {
            const files = state.files.filter((_, index) => !action.value.includes(index));
            return { ...state, files };
        }

        case "startRename":
            return { ...state, rename: { ...state.rename, renaming: true, rect: action.value.rect, inputValue: action.value.value } };

        case "udpateName":
            return { ...state, rename: { ...state.rename, inputValue: action.value } };

        case "rename": {
            const files = [...state.files];
            const index = files.findIndex((file) => file.id == action.value.id);
            if (index >= 0) {
                files[index] = action.value;
            }

            return { ...state, files };
        }

        case "endRename":
            return { ...state, rename: { ...state.rename, renaming: false } };

        case "startDrag":
            return { ...state, dragState: { ...state.dragState, dragging: true, startId: action.value.startId, dir: action.value.dir } };

        case "drag":
            return { ...state, dragState: { ...state.dragState, targetId: action.value } };

        case "endDrag":
            return { ...state, dragState: { ...state.dragState, dragging: false, startId: "", dir: "", targetId: "" } };

        case "preventBlur":
            return { ...state, preventBlur: action.value };

        case "toggleShuffle":
            return { ...state, shuffle: !state.shuffle };

        case "toggleSearch":
            return { ...state, searchState: { ...state.searchState, searching: action.value } };

        case "highlightItems":
            return { ...state, searchState: { ...state.searchState, itemIds: action.value, highlighIndex: 0 } };

        case "changeHighlight":
            return { ...state, searchState: { ...state.searchState, highlighIndex: action.value } };

        case "startMove":
            return { ...state, moveState: { ...state.moveState, started: true, progress: 0 } };

        case "endMove":
            return { ...state, moveState: { ...state.moveState, started: false, progress: 0 } };

        case "moveProgress":
            return { ...state, moveState: { ...state.moveState, progress: action.value } };

        default:
            return state;
    }
};

const store = writable(initialAppState);

export const dispatch = (action: AppAction) => {
    store.update((state) => updater(state, action));
};

export const appState = store;
