<script lang="ts">
    import { onMount } from "svelte";
    import MenuItem from "./MenuItem.svelte";
    import { appState, dispatch } from "./appStateReducer";

    export let type:Mp.ContextMenuType;

    const menuItemHeight = 27;
    const separatorHeight = 11;
    const itemMargin = 60;
    const fontSize = 12;
    const topBottomBorder = 2;
    const menuPadding = 10 + topBottomBorder;

    const getMenuWidth = (item:Mp.ContextMenuItem) => {
        const length = item.label ? item.label.length : 1;
        return (length * fontSize) + itemMargin
    }

    const calculateSize = () => {

        if(!$appState.items[type].length) return;

        let menuHeight = menuPadding;
        let menuWidth = 0;
        let submenuWidth = 0;
        let extraItemCount = 0;

        const items = $appState.items[type];

        items.forEach((item, index) => {
            const height = item.kind == "separator" ? separatorHeight : menuItemHeight
            menuHeight += height;
            menuWidth = Math.max(menuWidth, getMenuWidth(item))

            if(item.kind == "submenu"){
                const submenu = item.submenu ?? [];
                const subMenuCount = (submenu.length + index) - items.length
                if(subMenuCount > 0){
                    extraItemCount = Math.max(extraItemCount, subMenuCount)
                }
                submenu.forEach(item => {
                    submenuWidth = Math.max(submenuWidth, getMenuWidth(item))
                })
            }
        })

        menuHeight += menuItemHeight * extraItemCount;
        menuWidth += submenuWidth
        console.log(menuHeight)
        console.log(menuWidth)
        const size = {width:menuWidth, height:menuHeight};
        dispatch({type:"setMenuSize", value:{type, size}})

    }

    onMount(() => {
        calculateSize()
    })

</script>
<div class="menu-container" class:revert={$appState.revert} style={$appState.revert ? `transform:translateX(${window.outerWidth - $appState.menuSizes[type].width}px)` : "transform:translateX(0)"}>
    {#each $appState.items[type] as item}
        {#if item.kind == "submenu"}
            <div class="submenu-container menu-item">
                <div class="submenu-header">{item.label}</div>
                <div class="submenu">
                    {#each item.submenu ?? [] as submenuItem}
                        <MenuItem item={submenuItem}/>
                    {/each}
                </div>
            </div>
        {:else}
            <MenuItem item={item}/>
        {/if}
    {/each}
</div>
