<script lang="ts">
    import { onMount } from "svelte";
    import {getPlayerContextMenu, getPlaylistContextMenu} from "./menuitems"
    import Menu from "./Menu.svelte";
    import { IPC } from "../ipc";
    import settings from "../settings";
    import { LogicalPosition, LogicalSize, getCurrent } from "@tauri-apps/api/window";
    import { appState, dispatch } from "./appStateReducer";

    const ipc = new IPC("ContextMenu");
    const bound = {
        right:screen.availWidth,
        bottom:screen.availHeight
    }

    const prepare = (e:Mp.ReadyEvent) => {
        console.log(e)
        settings.init(e.settings);
        dispatch({type:"settings"})
        dispatch({type:"playerMenu", value:getPlayerContextMenu(settings.data)})
        dispatch({type:"playlistMenu", value:getPlaylistContextMenu()})
    }

    const getPosition = (menuType:Mp.ContextMenuType, mousePosition:Mp.Position) => {

        const size = $appState.menuSizes[menuType];

        const position = {
            x:mousePosition.x,
            y:mousePosition.y,
        }

        const right = mousePosition.x + size.width
        const bottom = mousePosition.y + size.height

        if(bottom >= bound.bottom) position.y = position.y - size.height;

        if(right >= bound.right){
            position.x = position.x - size.width;
            dispatch({type:"revert", value:true})
        }else{
            dispatch({type:"revert", value:false})
        }

        return position;
    }

    const popup = async (e:Mp.PopupContextMenuRequest) => {
        await ipc.invoke("change_theme", "dark")
        dispatch({type:"menu", value:e.type})
        const size = $appState.menuSizes[e.type];
        const position = getPosition(e.type, e.position)

        const window = getCurrent();
        await window.setPosition(new LogicalPosition(position.x, position.y ))
        await window.setSize(new LogicalSize(size.width, size.height))
        await window.show();
    }

    onMount(() => {
        ipc.receive("ready", prepare);
        ipc.receive("open-context-menu", popup);

        return () => {
            ipc.release()
        };
    })

</script>

{#if $appState.menuType == "Player"}
    <Menu type={"Player"}/>
{/if}

{#if $appState.menuType == "Playlist"}
    <Menu type={"Playlist"}/>
{/if}