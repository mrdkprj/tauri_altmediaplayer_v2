<script lang="ts">
    export let item:Mp.ContextMenuItem;
</script>
{#if item.kind === "separator"}
    <div class="menu-separator"></div>
{:else}
    <div class="menu-item" class:checkbox-menu={item.kind == "checkbox"} data-name={item.name} data-type={item.kind} data-value={item.value}>
        <div style="pointer-events: none;">{item.label}</div>
        <div style="width: 40px;"></div>
        <div style="pointer-events: none;">{item.accelerator ?? ""}</div>
    </div>
{/if}


