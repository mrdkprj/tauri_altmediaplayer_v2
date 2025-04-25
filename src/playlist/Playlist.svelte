<script lang="ts">
    import { onMount } from "svelte";
    import List from "./List.svelte";

    import editor from "./editor";
    import { getDropFiles, getTauriDropFiles } from "../fileDropHandler";
    import { handleShortcut } from "../shortcut";
    import { handleKeyEvent, Buttons, EmptyFile, PLATFROMS } from "../constants";
    import { appState, dispatch } from "./appStateReducer";
    import { t, lang } from "../translation/useTranslation";
    import { IPC } from "../ipc";
    import util from "../util";
    import Deferred from "../deferred";
    import path from "../path";

    import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

    let openContextMenu = false;
    let fileListContainer: HTMLDivElement;
    let randomIndices: number[] = [];
    let fileReleasePromise: Deferred<Mp.ReleaseFileResult>;

    const ipc = new IPC("Playlist");
    const List_Item_Padding = 10;

    const onContextMenu = async (e: MouseEvent) => {
        e.preventDefault();
        if (navigator.userAgent.includes(PLATFROMS.linux)) {
            openContextMenu = true;
        } else {
            await ipc.invoke("open_context_menu", { x: e.screenX, y: e.screenY });
        }
    };

    const openSortMenu = async (e: MouseEvent) => {
        await ipc.invoke("open_sort_context_menu", { x: e.screenX, y: e.screenY - 150 });
    };

    const onMouseUp = async (e: MouseEvent) => {
        if (navigator.userAgent.includes(PLATFROMS.linux)) {
            if (e.button == 2 && e.buttons == 0 && openContextMenu) {
                await ipc.invoke("open_context_menu", { x: e.clientX, y: e.clientY });
                openContextMenu = false;
            }
        }
    };

    const onPlaylistItemMousedown = (e: MouseEvent) => {
        if (!e.target || !(e.target instanceof HTMLElement)) return;

        if (e.button === Buttons.right && $appState.selection.selectedIds.length > 1) {
            if ($appState.selection.selectedIds.includes(e.target.id)) {
                return;
            }
        }

        return toggleSelect(e);
    };

    const onDrop = (e: DragEvent) => {
        if ($appState.dragState.dragging) return;

        e.preventDefault();
        e.stopPropagation();

        if (navigator.userAgent.includes(PLATFROMS.windows)) {
            if (e.dataTransfer && e.dataTransfer.files) {
                window.chrome.webview.postMessageWithAdditionalObjects("getPathForFiles", e.dataTransfer.files);
            }
        }
    };

    const onFileDrop = async (e: Mp.FileDropEvent) => {
        if ($appState.dragState.dragging) return;

        const files = getDropFiles(e);

        if (files.length) {
            await addToPlaylist(files);
        }
    };

    const onTauriFileDrop = async (e: Mp.TauriFileDropEvent) => {
        if ($appState.dragState.dragging) return;

        const files = getTauriDropFiles(e);

        if (files.length) {
            await addToPlaylist(files);
        }
    };

    const onPlaylistItemClicked = async (id: string) => {
        const index = getChildIndex(id);
        dispatch({ type: "currentIndex", value: index });
        await loadMediaFile(true);
    };

    const loadMediaFile = async (autoPlay: boolean, startFrom?: number) => {
        const currentFile = getCurrentFile();
        if (currentFile.id) {
            select(currentFile.id);
        }
        await ipc.sendTo("Player", "load-file", { currentFile, type: autoPlay ? "Play" : "Load", startFrom });
    };

    const getCurrentFile = () => {
        if ($appState.currentIndex < 0) return EmptyFile;

        if (!$appState.files.length) return EmptyFile;

        return $appState.files[$appState.currentIndex];
    };

    const reset = () => {
        randomIndices.length = 0;
        dispatch({ type: "clear" });
    };

    const initPlaylist = async (e: Mp.LoadPlaylistEvent) => {
        const fullPaths = e.files;

        reset();

        const files = await Promise.all(fullPaths.map(async (fullPath) => await util.toFile(fullPath)));

        dispatch({ type: "files", value: files });

        if (!files.length) return;

        dispatch({ type: "currentIndex", value: 0 });

        sortPlayList();

        shuffleList();

        await loadMediaFile(true);
    };

    const addToPlaylist = async (fullPaths: string[]) => {
        const paths = fullPaths.filter((fullPath) => $appState.files.findIndex((file) => file.fullPath == fullPath) < 0);
        const newFiles = await util.toFiles(paths);

        dispatch({ type: "appendFiles", value: newFiles });

        sortPlayList();

        shuffleList();

        if ($appState.files.length && $appState.currentIndex < 0) {
            dispatch({ type: "currentIndex", value: 0 });
            await loadMediaFile(false);
        }
    };

    const getRandomIndex = (value: number) => {
        if (value > 0) {
            randomIndices.unshift($appState.currentIndex);
            return randomIndices.pop() as number;
        }

        randomIndices.push($appState.currentIndex);
        return randomIndices.shift() as number;
    };

    const changeIndex = async (e: Mp.ChangePlaylistRequest) => {
        let nextIndex = $appState.shuffle ? getRandomIndex(e.index) : $appState.currentIndex + e.index;

        if (nextIndex >= $appState.files.length) {
            nextIndex = 0;
        }

        if (nextIndex < 0) {
            nextIndex = $appState.files.length - 1;
        }

        dispatch({ type: "currentIndex", value: nextIndex });

        await loadMediaFile(false);
    };

    const sortPlayList = () => {
        if (!$appState.files.length) return;

        const currentFileId = getCurrentFile().id;

        if ($appState.sortType.groupBy) {
            util.sortByGroup($appState.files, $appState.sortType.order);
        } else {
            util.sort($appState.files, $appState.sortType.order);
        }

        dispatch({ type: "files", value: $appState.files });

        const sortedIds = $appState.files.map((file) => file.id);

        if (currentFileId) {
            const index = sortedIds.findIndex((id) => id === currentFileId);
            dispatch({ type: "currentIndex", value: index });
        }
    };

    const shuffleList = () => {
        if (!$appState.shuffle) return;

        const target = new Array($appState.files.length)
            .fill(undefined)
            .map((_v, i) => i)
            .filter((i) => i !== $appState.currentIndex);
        randomIndices = util.shuffle(target);
    };

    const changePlaylistItemOrder = (data: Mp.ChangePlaylistOrderRequet) => {
        if (data.start === data.end) return;

        const currentId = getCurrentFile().id;

        const replacing = $appState.files.splice(data.start, 1)[0];
        $appState.files.splice(data.end, 0, replacing);

        const currentIndex = $appState.files.findIndex((file) => file.id == currentId);
        dispatch({ type: "currentIndex", value: currentIndex });
        dispatch({ type: "files", value: $appState.files });
    };

    const clearPlaylist = async () => {
        dispatch({ type: "clear" });
        await loadMediaFile(false);
    };

    const clearSelection = () => {
        dispatch({ type: "clearSelection" });
    };

    const getChildIndex = (id: string | null | undefined) => {
        return $appState.files.findIndex((file) => file.id == id);
    };

    const scrollToElement = (id: string) => {
        const element = document.getElementById(id);

        if (!element) return;

        const rect = element.getBoundingClientRect();
        const containerRect = fileListContainer.getBoundingClientRect();
        if (rect.top <= containerRect.top) {
            element.scrollIntoView(true);
        }

        if (rect.bottom > containerRect.height + containerRect.top + 5) {
            element.scrollIntoView(false);
        }
    };

    const removeFromPlaylist = async (autoPlay = false) => {
        if (!$appState.selection.selectedIds.length) return;

        const removeIndices = $appState.files.filter((file) => $appState.selection.selectedIds.includes(file.id)).map((file) => $appState.files.indexOf(file));
        const isCurrentFileRemoved = removeIndices.includes($appState.currentIndex);

        const selectedIndex = $appState.files.findIndex((file) => file.id == $appState.selection.selectedId);
        const shouldRestoreSelection = $appState.selection.selectedIds.length == 1;

        dispatch({ type: "removeFiles", value: removeIndices });

        clearSelection();

        if (shouldRestoreSelection && $appState.files.length) {
            const nextId = selectedIndex > $appState.files.length - 1 ? $appState.files[selectedIndex - 1].id : $appState.files[selectedIndex].id;
            dispatch({ type: "updateSelection", value: { selectedId: nextId, selectedIds: [nextId] } });
        }

        dispatch({ type: "currentIndex", value: getIndexAfterRemove(removeIndices) });

        if (isCurrentFileRemoved) {
            await loadMediaFile(autoPlay);
        }
    };

    const getIndexAfterRemove = (removeIndices: number[]) => {
        if (removeIndices.includes($appState.currentIndex)) {
            if (!$appState.files.length) return -1;

            const nextIndex = removeIndices[0];

            if (nextIndex >= $appState.files.length) {
                return $appState.files.length - 1;
            }

            return nextIndex;
        }

        if (removeIndices[0] < $appState.currentIndex) {
            return $appState.currentIndex - removeIndices.length;
        }

        return $appState.currentIndex;
    };

    const trash = async () => {
        if (!$appState.selection.selectedIds.length) return;

        const result = await releaseFile($appState.selection.selectedIds);

        try {
            const targetFilePaths = $appState.files.filter((file) => $appState.selection.selectedIds.includes(file.id)).map((file) => file.fullPath);

            if (!targetFilePaths.length) return;

            await Promise.all(targetFilePaths.map(async (item) => await ipc.invoke("trash", item)));

            await removeFromPlaylist(result.playing);
        } catch (ex: any) {
            await util.showErrorMessage(ex);
        }
    };

    const moveFile = async () => {
        const files = $appState.files.filter((file) => $appState.selection.selectedIds.includes(file.id));

        if (!files.length) return;

        const sourceDirs = new Set(files.map((file) => file.dir));

        if (sourceDirs.size > 1) return;

        if (files.length > 1) {
            const confimed = await ipc.invoke("message", { dialog_type: "ask", message: "Move multiple files. Are you sure?", title: "Move", kind: "warning", buttons: ["Yes", "No"] });
            if (!confimed) return;
        }

        const settings = await ipc.getSettings();

        const defaultPath = settings.defaultPath ? settings.defaultPath : files[0].dir;
        const result = await ipc.invoke("open", { default_path: defaultPath, properties: ["OpenDirectory"] });

        if (!result.file_paths.length) return;

        const destPath = result.file_paths[0];
        try {
            await releaseFile($appState.selection.selectedIds);

            settings.defaultPath = destPath;

            await ipc.updateSettings(settings);

            dispatch({ type: "startMove" });

            const sourcePaths = files.map((file) => file.fullPath);

            await ipc.invoke("mv_all", { from: sourcePaths, to: destPath });

            dispatch({ type: "endMove" });

            await removeFromPlaylist();
        } catch (ex: any) {
            await util.showErrorMessage(ex);
            dispatch({ type: "endMove" });
        }
    };

    const onFileMoveProgress = (e: Mp.MoveProgressEvent) => {
        dispatch({ type: "moveProgress", value: e.transferred });
    };

    const toggleSelect = (e: MouseEvent) => {
        const id = (e.target as HTMLElement).id;

        if (e.ctrlKey) {
            selectByCtrl(id);
            return;
        }

        if (e.shiftKey) {
            selectByShift(id);
            return;
        }

        selectByClick(id);
    };

    const select = (id: string) => {
        clearSelection();

        dispatch({ type: "updateSelection", value: { selectedId: id, selectedIds: [id] } });

        scrollToElement(id);
    };

    const selectByClick = (id: string) => {
        select(id);
    };

    const selectByShift = (id: string) => {
        dispatch({ type: "setSelectedIds", value: [] });

        const range = [];

        if ($appState.selection.selectedId) {
            range.push(getChildIndex($appState.selection.selectedId));
        } else {
            range.push(0);
        }

        range.push(getChildIndex(id));

        range.sort((a, b) => a - b);

        const ids: string[] = [];
        for (let i = range[0]; i <= range[1]; i++) {
            ids.push($appState.files[i].id);
        }

        dispatch({ type: "setSelectedIds", value: ids });
    };

    const selectByCtrl = (id: string) => {
        if (!$appState.selection.selectedId) {
            selectByClick(id);
            return;
        }

        dispatch({ type: "updatSelectedIds", value: [id] });
    };

    const selectAll = () => {
        clearSelection();

        const ids = $appState.files.map((file) => file.id);

        dispatch({ type: "updatSelectedIds", value: ids });
    };

    const moveSelectionByShit = (key: string) => {
        if (!$appState.selection.selectedIds.length) {
            select($appState.files[0].id);
        }

        const downward = $appState.selection.selectedId == $appState.selection.selectedIds[0];

        const currentId = downward ? $appState.selection.selectedIds[$appState.selection.selectedIds.length - 1] : $appState.selection.selectedIds[0];
        const currentIndex = getChildIndex(currentId);
        const nextId = key === "ArrowDown" ? $appState.files[currentIndex + 1]?.id : $appState.files[currentIndex - 1]?.id;

        if (!nextId) return;

        return selectByShift(nextId);
    };

    const moveSelection = (e: KeyboardEvent) => {
        if (!$appState.files.length) return;

        if (e.shiftKey) {
            return moveSelectionByShit(e.key);
        }

        const currentId = $appState.selection.selectedId ? $appState.selection.selectedId : $appState.files[0].id;
        const currentIndex = getChildIndex(currentId);
        const nextId = e.key === "ArrowDown" ? $appState.files[currentIndex + 1]?.id : $appState.files[currentIndex - 1]?.id;

        if (!nextId) return;

        clearSelection();
        select(nextId);
    };

    const moveSelectionUpto = (e: KeyboardEvent) => {
        if (!$appState.files.length) return;

        e.preventDefault();

        const nextSelection = findNextSelection(e.key === "Home", e.ctrlKey);

        if (!nextSelection.selectId || !nextSelection.scrollToId) return;

        if (e.shiftKey) {
            selectByShift(nextSelection.selectId);
        } else {
            select(nextSelection.selectId);
        }
        scrollToElement(nextSelection.scrollToId);
    };

    const findNextSelection = (upward: boolean, ctrlKey: boolean): Mp.MoveUptoSelection => {
        const defaultTarget = upward ? $appState.files[0] : $appState.files[$appState.files.length - 1];

        const nextSelection: Mp.MoveUptoSelection = { selectId: defaultTarget.id, scrollToId: $appState.sortType.groupBy ? defaultTarget.id : encodeURIComponent(defaultTarget.dir) };

        const dirs = Array.from(document.querySelectorAll(".group"));

        if (dirs.length <= 1 || !$appState.sortType.groupBy || ctrlKey) {
            return nextSelection;
        }

        const current = document.getElementById($appState.selection.selectedId);
        const currentIndex = $appState.files.findIndex((file) => file.id == $appState.selection.selectedId);

        if (!current || currentIndex < 0) return nextSelection;

        const nearest = upward ? current.previousElementSibling : current.nextElementSibling;

        if (!nearest) return nextSelection;

        const currentFile = $appState.files[currentIndex];
        let dir = currentFile.dir;

        if (nearest.classList.contains("group")) {
            if (upward && currentIndex > 0) {
                dir = $appState.files[currentIndex - 1].dir;
            }
            if (!upward && currentIndex < $appState.files.length - 1) {
                dir = $appState.files[currentIndex + 1].dir;
            }
        }

        const targetIndex = upward ? $appState.files.findIndex((file) => file.dir == dir) : $appState.files.findLastIndex((file) => file.dir == dir);

        if (targetIndex < 0) return nextSelection;

        const target = $appState.files[targetIndex];
        let scrollToId = target.id;
        if (upward) {
            scrollToId = encodeURIComponent(target.dir);
        }
        if (!upward && targetIndex < $appState.files.length - 1) {
            scrollToId = encodeURIComponent($appState.files[targetIndex + 1].dir);
        }

        return { selectId: target.id, scrollToId };
    };

    const setRenameInputFocus = (node: HTMLInputElement) => {
        node.focus();
        node.setSelectionRange(0, node.value.lastIndexOf("."));
    };

    const setSearchInputFocus = (node: HTMLInputElement) => {
        node.focus();
    };

    const onRenameInputKeyDown = async (e: KeyboardEvent) => {
        if (!e.target || !(e.target instanceof HTMLInputElement)) return;

        if ($appState.rename.renaming && e.key === "Enter") {
            e.stopPropagation();
            e.preventDefault();
            await endEditFileName();
        }
    };

    const startEditFileName = () => {
        const selectedElement = document.getElementById($appState.selection.selectedId);

        if (!selectedElement) return;

        const fileName = selectedElement.getAttribute("data-name") ?? "";

        const rect = selectedElement.getBoundingClientRect();

        editor.begin(selectedElement.id, fileName);
        dispatch({
            type: "startRename",
            value: {
                rect: {
                    top: rect.top,
                    left: rect.left,
                    width: selectedElement.offsetWidth - List_Item_Padding,
                    height: selectedElement.offsetHeight - List_Item_Padding,
                },
                value: fileName,
            },
        });

        dispatch({ type: "preventBlur", value: false });
    };

    const endRename = () => {
        dispatch({ type: "endRename" });
    };

    const endEditFileName = async () => {
        if (editor.data.name === $appState.rename.inputValue) {
            endRename();
        } else {
            editor.update($appState.rename.inputValue);
            await requestRename();
        }
    };

    const releaseFile = async (fileIds: string[]) => {
        ipc.sendTo("Player", "release-file", { fileIds });
        fileReleasePromise = new Deferred();
        return await fileReleasePromise.promise;
    };

    const onReleaseFile = (data: Mp.ReleaseFileResult) => {
        fileReleasePromise.resolve(data);
    };

    const requestRename = async () => {
        dispatch({ type: "preventBlur", value: true });

        const releaseResult = await releaseFile([editor.data.id]);

        const fileIndex = $appState.files.findIndex((file) => file.id == editor.data.id);
        const file = $appState.files[fileIndex];
        const oldPath = file.fullPath;
        const newPath = path.join(path.dirname(oldPath), editor.data.name);

        try {
            const fileExsists = await ipc.invoke("exists", newPath);
            if (fileExsists) {
                throw new Error(`File name "${editor.data.name}" exists`);
            }

            await ipc.invoke("rename", { old: oldPath, new: newPath });

            const newMediaFile = await util.updateFile(newPath, file);
            dispatch({ type: "rename", value: newMediaFile });

            editor.end();
        } catch (ex: any) {
            await util.showErrorMessage(ex);
            editor.rollback();
            select(editor.data.id);
        } finally {
            if (fileIndex == $appState.currentIndex) {
                await loadMediaFile(releaseResult.playing, releaseResult.currentTime);
            }
            endRename();
        }
    };

    const undoRename = async () => {
        if (!editor.canUndo()) return;

        editor.undo();

        select(editor.data.id);

        await requestRename();
    };

    const redoRename = async () => {
        if (!editor.canRedo()) return;

        editor.redo();

        select(editor.data.id);

        await requestRename();
    };

    /* Search */
    const toggleSearch = (forceClose = false) => {
        const showSearchBar = forceClose ? false : !$appState.searchState.searching;
        if (showSearchBar && $appState.rename.renaming) {
            return;
        }

        dispatch({ type: "toggleSearch", value: showSearchBar });

        if (showSearchBar) {
            onSearchInput();
        } else {
            dispatch({ type: "highlightItems", value: [] });
        }
    };

    const onSearchInput = (e?: Event & { currentTarget: EventTarget & HTMLInputElement }) => {
        if (!$appState.files.length) return;

        const { value } = e?.target as HTMLInputElement;

        if (value) {
            const items = $appState.files.filter((file) => file.name.toLowerCase().includes(value.toLowerCase())).map((file) => file.id);
            dispatch({ type: "highlightItems", value: items });
        } else {
            dispatch({ type: "highlightItems", value: [] });
        }
    };

    const moveHighlight = (forward: boolean) => {
        if (!$appState.searchState.itemIds.length) return;

        if (forward) {
            const next = $appState.searchState.highlighIndex + 1;
            if (next > $appState.searchState.itemIds.length - 1) return;
            dispatch({ type: "changeHighlight", value: next });
        } else {
            const prev = $appState.searchState.highlighIndex - 1;
            if (prev < 0) return;
            dispatch({ type: "changeHighlight", value: prev });
        }

        const targetId = $appState.searchState.itemIds[$appState.searchState.highlighIndex];
        scrollToElement(targetId);
    };

    const toggleShuffle = () => {
        dispatch({ type: "toggleShuffle" });
        shuffleList();
    };

    const changeSortOrder = async (sortOrder: Mp.SortOrder) => {
        dispatch({ type: "sortType", value: { order: sortOrder, groupBy: $appState.sortType.groupBy } });
        sortPlayList();
        const settings = await ipc.getSettings();
        settings.sort.order = sortOrder;
        await ipc.updateSettings(settings);
    };

    const toggleGroupBy = async () => {
        dispatch({ type: "sortType", value: { order: $appState.sortType.order, groupBy: !$appState.sortType.groupBy } });
        const settings = await ipc.getSettings();
        settings.sort.groupBy = $appState.sortType.groupBy;
        await ipc.updateSettings(settings);
    };

    const copyFileNameToClipboard = async (fullPath: boolean) => {
        if (!$appState.selection.selectedIds.length) return;

        const files = $appState.files.filter((file) => $appState.selection.selectedIds.includes(file.id));

        if (!files) return;

        const names = files.map((file) => (fullPath ? file.fullPath : file.name));

        await ipc.invoke("write_text", names.join("\n"));
    };

    const pasteFilePath = async () => {
        const paths = await ipc.invoke("read_text", undefined);
        const lineBreak = paths.includes("\r") ? "\r\n" : "\n";

        if (paths) {
            const files = await Promise.all(
                paths
                    .split(lineBreak)
                    .filter(Boolean)
                    .filter(async (fullPath) => await ipc.invoke("exists", fullPath)),
            );

            if (files.length) {
                await addToPlaylist(files);
            }
        }
    };

    const displayMetadata = async () => {
        const file = $appState.files.find((file) => file.id == $appState.selection.selectedId);
        if (!file) return;

        const metadata = await util.getMediaMetadata(file.fullPath);
        const metadataString = JSON.stringify(metadata, undefined, 2).replaceAll('"', "");
        const result = await ipc.invoke("message", { dialog_type: "confirm", message: metadataString, kind: "info", buttons: ["OK", "Copy"], cancel_id: 1 });
        if (!result) {
            await ipc.invoke("write_text", metadataString);
        }
    };

    const reveal = async () => {
        if (!$appState.selection.selectedId) return;

        const file = $appState.files.find((file) => file.id == $appState.selection.selectedId);

        if (!file) return;

        await ipc.invoke("reveal", { file_path: file.fullPath, use_file_manager: $appState.useFileManager });
    };

    const toggleUseFileManager = async () => {
        dispatch({ type: "toggleUseFileManager" });
        const settings = await ipc.getSettings();
        settings.useDefaultFileManager = $appState.useFileManager;
        await ipc.updateSettings(settings);
    };

    const openConvert = async (opener: Mp.DialogOpener) => {
        const file = $appState.files.find((file) => file.id == $appState.selection.selectedId) ?? EmptyFile;
        await ipc.sendTo("Convert", "open-convert", { file, opener });
    };

    const onKeydown = async (e: KeyboardEvent) => {
        if ($appState.rename.renaming) return;

        if ($appState.searchState.searching) {
            if (e.key === "Escape") {
                e.preventDefault();
                toggleSearch(true);
            }

            if (e.key === "F3") {
                e.preventDefault();
                if (e.shiftKey) {
                    moveHighlight(false);
                } else {
                    moveHighlight(true);
                }
            }

            return;
        }

        if (e.key == "F5") {
            e.preventDefault();
            return;
        }

        if (e.key === "Enter") {
            e.preventDefault();
            return await ipc.send("toggle-play", {});
        }

        if (e.ctrlKey && e.key === "a") {
            e.preventDefault();
            return selectAll();
        }

        if (e.ctrlKey && e.key === "z") {
            e.preventDefault();
            return await undoRename();
        }

        if (e.ctrlKey && e.key === "y") {
            e.preventDefault();
            return await redoRename();
        }

        if (e.key === "ArrowUp" || e.key === "ArrowDown") {
            e.preventDefault();
            return moveSelection(e);
        }

        if (e.key === "Home" || e.key === "End") {
            e.preventDefault();
            return moveSelectionUpto(e);
        }

        if (e.ctrlKey && e.key === "f") {
            e.preventDefault();
            return toggleSearch();
        }

        const shortcut = handleShortcut("Playlist", e);
        if (shortcut) {
            return await onContextMenuSelect(shortcut);
        }
    };

    const onContextMenuSelect = async (e: Mp.ContextMenuEvent) => {
        const id = e.name ? e.name : e.id;
        await handleContextMenu(id as keyof Mp.PlaylistContextMenuSubTypeMap, e.id as keyof Mp.PlaylistContextMenuSubTypeMap);
    };

    const handleContextMenu = async (menuId: keyof Mp.PlaylistContextMenuSubTypeMap, value: keyof Mp.PlaylistContextMenuSubTypeMap) => {
        switch (menuId) {
            case "Remove":
                await removeFromPlaylist();
                break;
            case "RemoveAll":
                await clearPlaylist();
                break;
            case "Trash":
                trash();
                break;
            case "CopyFileName":
                await copyFileNameToClipboard(false);
                break;
            case "CopyFullpath":
                await copyFileNameToClipboard(true);
                break;
            case "Reveal":
                await reveal();
                break;
            case "Metadata":
                await displayMetadata();
                break;
            case "Convert":
                await openConvert("user");
                break;
            case "Sort":
                await changeSortOrder(value as Mp.SortOrder);
                break;
            case "Rename":
                startEditFileName();
                break;
            case "Move":
                await moveFile();
                break;
            case "GroupBy":
                await toggleGroupBy();
                break;
            case "PasteFilePath":
                await pasteFilePath();
                break;
            case "useDefaultFileManager":
                await toggleUseFileManager();
                break;
        }
    };

    const close = async () => {
        await ipc.sendTo("Player", "toggle-playlist-visible", {});
        await WebviewWindow.getCurrent().hide();
    };

    const prepare = async () => {
        const settings = await ipc.getSettings();

        $lang = settings.locale.lang;

        dispatch({ type: "sortType", value: { order: settings.sort.order, groupBy: settings.sort.groupBy } });

        const playlist = WebviewWindow.getCurrent();

        await playlist.setPosition(util.toPhysicalPosition(settings.playlistBounds));

        await playlist.setSize(util.toPhysicalSize(settings.playlistBounds));

        if (settings.playlistVisible) {
            await playlist.show();
        }
    };

    onMount(() => {
        prepare();
        ipc.receive("contextmenu-event", onContextMenuSelect);
        ipc.receive("load-playlist", initPlaylist);
        if (navigator.userAgent.includes(PLATFROMS.linux)) {
            ipc.receiveTauri<Mp.TauriFileDropEvent>("tauri://drag-drop", onTauriFileDrop);
        } else {
            window.chrome.webview.addEventListener("message", onFileDrop);
        }
        ipc.receive("change-playlist", changeIndex);
        ipc.receive("restart", clearPlaylist);
        ipc.receive("file-released", onReleaseFile);
        ipc.receive("move-progress", onFileMoveProgress);

        return () => {
            if (navigator.userAgent.includes(PLATFROMS.windows)) {
                window.chrome.webview.removeEventListener("message", onFileDrop);
            }
            ipc.release();
        };
    });
</script>

<svelte:window oncontextmenu={onContextMenu} onkeydown={onKeydown} onmouseup={onMouseUp} />

<div class="viewport">
    <div class="title-bar">
        <div class="close-btn" onclick={close} onkeydown={handleKeyEvent} role="button" tabindex="-1">&times;</div>
    </div>

    <div
        class="playlist-viewport"
        class:group-by={$appState.sortType.groupBy}
        bind:this={fileListContainer}
        role="button"
        tabindex="-1"
        onscroll={endEditFileName}
        ondragover={(e) => e.preventDefault()}
        ondrop={onDrop}
    >
        {#if $appState.rename.renaming}
            <input
                type="text"
                class="input rename"
                style="top:{$appState.rename.rect.top}px; left:{$appState.rename.rect.left}px; width:{$appState.rename.rect.width}px; height:{$appState.rename.rect.height}px"
                spellCheck="false"
                onblur={$appState.preventBlur ? undefined : endEditFileName}
                onkeydown={onRenameInputKeyDown}
                bind:value={$appState.rename.inputValue}
                use:setRenameInputFocus
            />
        {/if}
        {#if $appState.searchState.searching}
            <div class="searchArea">
                <input type="text" spellcheck="false" class="input search" oninput={onSearchInput} use:setSearchInputFocus />
                <span class="searchResult">{$appState.searchState.itemIds.length ? $appState.searchState.highlighIndex + 1 : 0}/{$appState.searchState.itemIds.length}</span>
            </div>
        {/if}
        <List {onPlaylistItemClicked} onEndDrag={changePlaylistItemOrder} onMouseDown={onPlaylistItemMousedown} {scrollToElement} {getChildIndex} />
    </div>
    <div class="playlist-footer" class:shuffle={$appState.shuffle}>
        <div class="btn shuffle-btn" title={$t("shuffle")} onclick={toggleShuffle} onkeydown={handleKeyEvent} role="button" tabindex="-1">
            <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                <path
                    fill-rule="evenodd"
                    d="M0 3.5A.5.5 0 0 1 .5 3H1c2.202 0 3.827 1.24 4.874 2.418.49.552.865 1.102 1.126 1.532.26-.43.636-.98 1.126-1.532C9.173 4.24 10.798 3 13 3v1c-1.798 0-3.173 1.01-4.126 2.082A9.624 9.624 0 0 0 7.556 8a9.624 9.624 0 0 0 1.317 1.918C9.828 10.99 11.204 12 13 12v1c-2.202 0-3.827-1.24-4.874-2.418A10.595 10.595 0 0 1 7 9.05c-.26.43-.636.98-1.126 1.532C4.827 11.76 3.202 13 1 13H.5a.5.5 0 0 1 0-1H1c1.798 0 3.173-1.01 4.126-2.082A9.624 9.624 0 0 0 6.444 8a9.624 9.624 0 0 0-1.317-1.918C4.172 5.01 2.796 4 1 4H.5a.5.5 0 0 1-.5-.5z"
                />
                <path
                    d="M13 5.466V1.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384l-2.36 1.966a.25.25 0 0 1-.41-.192zm0 9v-3.932a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384l-2.36 1.966a.25.25 0 0 1-.41-.192z"
                />
            </svg>
        </div>
        <div class="btn" title={$t("sort")} onclick={openSortMenu} onkeydown={handleKeyEvent} role="button" tabindex="-1">
            {#if $appState.sortType.order == "NameAsc"}
                <svg xmlns="http://www.w3.org/2000/svg" id="nameAsc" fill="currentColor" viewBox="0 0 16 16">
                    <path fill-rule="evenodd" d="M10.082 5.629 9.664 7H8.598l1.789-5.332h1.234L13.402 7h-1.12l-.419-1.371h-1.781zm1.57-.785L11 2.687h-.047l-.652 2.157h1.351z" />
                    <path
                        d="M12.96 14H9.028v-.691l2.579-3.72v-.054H9.098v-.867h3.785v.691l-2.567 3.72v.054h2.645V14zM4.5 2.5a.5.5 0 0 0-1 0v9.793l-1.146-1.147a.5.5 0 0 0-.708.708l2 1.999.007.007a.497.497 0 0 0 .7-.006l2-2a.5.5 0 0 0-.707-.708L4.5 12.293V2.5z"
                    />
                </svg>
            {/if}
            {#if $appState.sortType.order == "NameDesc"}
                <svg xmlns="http://www.w3.org/2000/svg" id="nameDesc" fill="currentColor" viewBox="0 0 16 16">
                    <path fill-rule="evenodd" d="M10.082 5.629 9.664 7H8.598l1.789-5.332h1.234L13.402 7h-1.12l-.419-1.371h-1.781zm1.57-.785L11 2.687h-.047l-.652 2.157h1.351z" />
                    <path
                        d="M12.96 14H9.028v-.691l2.579-3.72v-.054H9.098v-.867h3.785v.691l-2.567 3.72v.054h2.645V14zm-8.46-.5a.5.5 0 0 1-1 0V3.707L2.354 4.854a.5.5 0 1 1-.708-.708l2-1.999.007-.007a.498.498 0 0 1 .7.006l2 2a.5.5 0 1 1-.707.708L4.5 3.707V13.5z"
                    />
                </svg>
            {/if}
            {#if $appState.sortType.order == "DateAsc"}
                <svg xmlns="http://www.w3.org/2000/svg" id="dateAsc" fill="currentColor" viewBox="0 0 16 16">
                    <path d="M12.438 1.668V7H11.39V2.684h-.051l-1.211.859v-.969l1.262-.906h1.046z" />
                    <path
                        fill-rule="evenodd"
                        d="M11.36 14.098c-1.137 0-1.708-.657-1.762-1.278h1.004c.058.223.343.45.773.45.824 0 1.164-.829 1.133-1.856h-.059c-.148.39-.57.742-1.261.742-.91 0-1.72-.613-1.72-1.758 0-1.148.848-1.835 1.973-1.835 1.09 0 2.063.636 2.063 2.687 0 1.867-.723 2.848-2.145 2.848zm.062-2.735c.504 0 .933-.336.933-.972 0-.633-.398-1.008-.94-1.008-.52 0-.927.375-.927 1 0 .64.418.98.934.98z"
                    />
                    <path d="M4.5 2.5a.5.5 0 0 0-1 0v9.793l-1.146-1.147a.5.5 0 0 0-.708.708l2 1.999.007.007a.497.497 0 0 0 .7-.006l2-2a.5.5 0 0 0-.707-.708L4.5 12.293V2.5z" />
                </svg>
            {/if}
            {#if $appState.sortType.order == "DateDesc"}
                <svg xmlns="http://www.w3.org/2000/svg" id="dateDesc" fill="currentColor" viewBox="0 0 16 16">
                    <path d="M12.438 1.668V7H11.39V2.684h-.051l-1.211.859v-.969l1.262-.906h1.046z" />
                    <path
                        fill-rule="evenodd"
                        d="M11.36 14.098c-1.137 0-1.708-.657-1.762-1.278h1.004c.058.223.343.45.773.45.824 0 1.164-.829 1.133-1.856h-.059c-.148.39-.57.742-1.261.742-.91 0-1.72-.613-1.72-1.758 0-1.148.848-1.835 1.973-1.835 1.09 0 2.063.636 2.063 2.687 0 1.867-.723 2.848-2.145 2.848zm.062-2.735c.504 0 .933-.336.933-.972 0-.633-.398-1.008-.94-1.008-.52 0-.927.375-.927 1 0 .64.418.98.934.98z"
                    />
                    <path d="M4.5 13.5a.5.5 0 0 1-1 0V3.707L2.354 4.854a.5.5 0 1 1-.708-.708l2-1.999.007-.007a.498.498 0 0 1 .7.006l2 2a.5.5 0 1 1-.707.708L4.5 3.707V13.5z" />
                </svg>
            {/if}
        </div>
        {#if $appState.moveState.started}
            <div class="btn">
                <div class="loader8"></div>
            </div>
        {/if}
    </div>
</div>
