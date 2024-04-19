<script lang="ts">
    import { afterUpdate } from "svelte";
    import { appState, dispatch } from "./appStateReducer";

    export let onPlaylistItemClicked:(id:string) => void;
    export let onMouseDown:(e:MouseEvent) => void;
    export let scrollToElement:(id:string) => void;
    export let getChildIndex:(id:string | null | undefined) => number;

    let listSize = 0;

    afterUpdate(() => {
        if($appState.files.length != listSize){
            listSize = $appState.files.length
            scrollToElement($appState.selection.selectedId)
        }
    })

    const onItemClicked = (e:MouseEvent) => {
        onPlaylistItemClicked((e.target as HTMLElement).id)
    }

    const startDragPlaylistItem = (e:DragEvent) => {
        if(!e.target || !(e.target instanceof HTMLElement)) return;

        e.stopPropagation();
        dispatch({type:"startDrag", value:{startId:e.target.id, dir:e.target.getAttribute("data-dir") ?? ""}})
    }

    const toggleHighlightDropTarget = (e:DragEvent) => {

        if(!$appState.dragState.dragging) return;

        if(!e.target || !(e.target instanceof HTMLElement)) return;

        if($appState.dragState.dir !== e.target.getAttribute("data-dir")){
            $appState.dragState.targetId = "";
            return;
        }

        dispatch({type:"drag", value:e.target.id})

    }

    const endDragPlaylistItem = (_e:DragEvent) => {

        if(!$appState.dragState.dragging) return;

        if($appState.dragState.targetId){

            window.api.send("change-playlist-order", {
                start:getChildIndex($appState.dragState.startId),
                end:getChildIndex($appState.dragState.targetId),
                currentIndex:$appState.currentIndex
            });

        }

        dispatch({type:"endDrag"})
    }


</script>

<div id="fileList" class="file-list" class:grou-by={$appState.sortType.groupBy}>

    {#each $appState.files as file, index}

        {#if index == 0 || $appState.files[index - 1].dir != $appState.files[index].dir }
            <div class="group separator" title={file.dir} id={encodeURIComponent(file.dir)}>
                <div class="left separator"></div>
                <div class="mid separator">{file.dir}</div>
                <div class="right separator"></div>
            </div>
        {/if}

        <div
            title={file.name}
            id={file.id}
            draggable="true"
            class="playlist-item"
            class:current={$appState.currentIndex === index}
            class:selected={$appState.selection.selectedIds.includes(file.id)}
            class:draghover={$appState.dragState.targetId == file.id}
            data-dir={encodeURIComponent(file.dir)}
            on:mousedown={onMouseDown}
            on:dblclick={onItemClicked}
            on:dragstart={startDragPlaylistItem}
            on:dragenter={toggleHighlightDropTarget}
            on:dragend={endDragPlaylistItem}
            role="button"
            tabindex="-1"
        >{file.name}</div>

    {/each}
</div>