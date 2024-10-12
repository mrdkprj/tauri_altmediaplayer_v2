<script lang="ts">
    import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { IPC } from "../ipc";
    import { onMount } from "svelte";

    const ipc = new IPC("Tag");
    let tags: string[] = [];

    const deleteTag = (index: number) => {
        tags = tags.filter((_, i) => index !== i);
    };

    const addTag = () => {
        if (tags.includes("")) return;
        tags = [...tags, ""];
    };

    const save = async () => {
        const sortedTags = tags.sort().filter(Boolean);
        tags = [...new Set(sortedTags)];
        await ipc.sendTo("Player", "change-tags", tags);
        close();
    };

    const close = async () => {
        await WebviewWindow.getCurrent().hide();
    };

    const show = async () => {
        const settings = await ipc.invoke("get_settings", undefined);
        tags = settings.tags;
        await WebviewWindow.getCurrent().show();
    };

    const onKeydown = (e: KeyboardEvent) => {
        if (e.key === "Enter") {
            e.preventDefault();
            save();
        }

        if (e.key === "Escape") {
            e.preventDefault();
            close();
        }
    };

    const setFocus = (taget: HTMLDivElement, index: number) => {
        if (!tags[index]) {
            taget.focus();
        }
    };

    onMount(() => {
        ipc.receive("open-tag-editor", show);

        return () => {
            ipc.release();
        };
    });
</script>

<svelte:document on:keydown={onKeydown} />

<div class="viewport">
    <div class="title-bar">
        <div class="close-btn" on:click={close} on:keydown={onKeydown} role="button" tabindex="-1">&times;</div>
    </div>
    <div class="manager">
        <div class="tags">
            {#each tags as tag, index}
                <div class="tag">
                    <div class="tag-text" contenteditable="true" on:keydown={onKeydown} bind:textContent={tags[index]} use:setFocus={index} spellcheck="false" role="button" tabindex="-1">
                        {tag}
                    </div>
                    <div class="delete" on:click={() => deleteTag(index)} on:keydown={onKeydown} role="button" tabindex="-1">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                            <path
                                d="M6.5 1h3a.5.5 0 0 1 .5.5v1H6v-1a.5.5 0 0 1 .5-.5M11 2.5v-1A1.5 1.5 0 0 0 9.5 0h-3A1.5 1.5 0 0 0 5 1.5v1H1.5a.5.5 0 0 0 0 1h.538l.853 10.66A2 2 0 0 0 4.885 16h6.23a2 2 0 0 0 1.994-1.84l.853-10.66h.538a.5.5 0 0 0 0-1zm1.958 1-.846 10.58a1 1 0 0 1-.997.92h-6.23a1 1 0 0 1-.997-.92L3.042 3.5zm-7.487 1a.5.5 0 0 1 .528.47l.5 8.5a.5.5 0 0 1-.998.06L5 5.03a.5.5 0 0 1 .47-.53Zm5.058 0a.5.5 0 0 1 .47.53l-.5 8.5a.5.5 0 1 1-.998-.06l.5-8.5a.5.5 0 0 1 .528-.47M8 4.5a.5.5 0 0 1 .5.5v8.5a.5.5 0 0 1-1 0V5a.5.5 0 0 1 .5-.5"
                            />
                        </svg>
                    </div>
                </div>
            {/each}
            <div class="add tag">
                <button class="btn-sm" on:click={addTag}>+</button>
            </div>
        </div>
        <div class="buttons">
            <button class="btn" on:click={save}>Save</button>
            <button class="btn" on:click={close}>Close</button>
        </div>
    </div>
</div>
