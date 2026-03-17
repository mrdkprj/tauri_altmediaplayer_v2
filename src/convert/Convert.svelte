<script lang="ts">
    import { onMount } from "svelte";
    import RadioGroup from "./RadioGroup.svelte";

    import { AudioExtensions, VideoExtensions } from "../constants";
    import { appState } from "./appState.svelte";
    import { t } from "../translation/useTranslation.svelte";
    import { IPC } from "../ipc";
    import util from "../util";
    import path from "../path";

    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    const ipc = new IPC("Convert");

    const changeSourceFile = (file: Mp.MediaFile) => {
        appState.sourceFile = file.fullPath;
        const type = AudioExtensions.includes(file.extension.toLowerCase().replace(".", "")) ? "Audio" : "Video";
        appState.sourceType = type;
    };

    const closeDialog = async () => {
        await getCurrentWebviewWindow().hide();
    };

    const lock = () => {
        appState.converting = true;
        document.querySelectorAll("input").forEach((element) => (element.disabled = true));
    };

    const unlock = () => {
        appState.converting = false;
        document.querySelectorAll("input").forEach((element) => (element.disabled = false));
    };

    const requestConvert = async () => {
        if (!appState.sourceFile) return;

        lock();

        const args: Mp.ConvertRequest = {
            sourcePath: appState.sourceFile,
            convertType: appState.convertType,
            options: {
                format: appState.convertType == "Video" ? appState.videoCodec : appState.audioCodec,
                frameSize: appState.frameSize,
                audioBitrate: appState.audioBitrate,
                rotation: appState.rotation,
                audioVolume: appState.audioVolume,
                maxAudioVolume: appState.maxVolume,
            },
        };

        await startConvert(args);
    };

    const startConvert = async (data: Mp.ConvertRequest) => {
        const file = await util.toFile(data.sourcePath);

        const fileExists = await util.exists(file.fullPath);
        if (!fileExists) return endConvert();

        const extension = data.options.format.toLocaleLowerCase();
        const fileName = file.name.replace(path.extname(file.name), "");
        const defaultPath = path.join(file.dir, `${fileName}.${extension}`);
        const result = await ipc.invoke("save", {
            default_path: defaultPath,
            filters: [
                {
                    name: data.convertType,
                    extensions: [extension],
                },
            ],
        });

        if (!result.file_paths.length) return await endConvert();

        const selectedPath = result.file_paths[0];

        const shouldReplace = file.fullPath === selectedPath;

        const timestamp = String(new Date().getTime());
        const savePath = shouldReplace ? path.join(path.dirname(selectedPath), path.basename(selectedPath) + timestamp) : selectedPath;

        const webviewWindow = getCurrentWebviewWindow();
        await webviewWindow.hide();

        await ipc.sendTo("Player", "toggle-convert", {});

        try {
            if (data.convertType === "Video") {
                await util.convertVideo(data.sourcePath, savePath, data.options);
            } else {
                await util.convertAudio(data.sourcePath, savePath, data.options);
            }

            if (shouldReplace) {
                await ipc.invoke("rename", { new: selectedPath, old: savePath });
            }

            await endConvert();
        } catch (ex: any) {
            await endConvert(ex.message);
        } finally {
            await webviewWindow.show();
            await ipc.sendTo("Player", "toggle-convert", {});
        }
    };

    const endConvert = async (message?: any) => {
        if (message) {
            console.log(message);
            await util.showErrorMessage(JSON.stringify(message));
        }

        unlock();
    };

    const requestCancelConvert = async () => {
        await util.cancelConvert();
    };

    const openDialog = async () => {
        const result = await ipc.invoke("open", {
            default_path: path.dirname(appState.sourceFile),
            title: "Select file to convert",
            filters: [{ name: "Media File", extensions: VideoExtensions.concat(AudioExtensions) }],
            properties: ["OpenFile"],
        });

        if (!result.file_paths.length) return;

        const file = await util.toFile(result.file_paths[0]);
        if (VideoExtensions.concat(AudioExtensions).includes(file.extension)) {
            changeSourceFile(file);
        } else {
            await util.showErrorMessage(t("unsupportedMedia"));
        }
    };

    const onKeydown = async (e: KeyboardEvent) => {
        e.preventDefault();
        if (e.key === "Escape") {
            await closeDialog();
        }
    };

    const show = async (file: Mp.MediaFile) => {
        if (!appState.converting) {
            changeSourceFile(file);
        }
        await getCurrentWebviewWindow().show();
    };

    onMount(() => {
        ipc.receive("open-convert", show);

        return () => {
            ipc.release();
        };
    });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="viewport">
    <div data-tauri-drag-region={navigator.userAgent.includes("Linux") ? true : null} class="title-bar">
        <div class="close-btn" onclick={closeDialog} onkeydown={onKeydown} role="button" tabindex="-1">&times;</div>
    </div>
    <div class="convert-viewport">
        <div class="container">
            <div class="option-label">{t("inputFile")}</div>
            <div class="option-area">
                <div class="text">
                    <input type="text" class="source-file-input" readonly value={appState.sourceFile} />
                    <div class="btn" onclick={openDialog} onkeydown={onKeydown} role="button" tabindex="-1">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                            <path
                                d="M1 3.5A1.5 1.5 0 0 1 2.5 2h2.764c.958 0 1.76.56 2.311 1.184C7.985 3.648 8.48 4 9 4h4.5A1.5 1.5 0 0 1 15 5.5v7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9zM2.5 3a.5.5 0 0 0-.5.5V6h12v-.5a.5.5 0 0 0-.5-.5H9c-.964 0-1.71-.629-2.174-1.154C6.374 3.334 5.82 3 5.264 3H2.5zM14 7H2v5.5a.5.5 0 0 0 .5.5h11a.5.5 0 0 0 .5-.5V7z"
                            />
                        </svg>
                    </div>
                </div>
            </div>
            <div class="option-label">{t("convertType")}</div>
            <div class="option-area">
                <RadioGroup
                    options={["Video", "Audio"]}
                    labels={["Video", "Audio"]}
                    name="format"
                    checkedOption={appState.convertType}
                    bind:group={appState.convertType}
                    disableIf={{ condition: appState.sourceType == "Audio", target: "Video" }}
                />
            </div>
            <div class="option-label">Format</div>
            <div class="option-area">
                {#if appState.convertType == "Video"}
                    <select bind:value={appState.videoCodec}>
                        {#each VideoExtensions as format}
                            <option value={format}>{format}</option>
                        {/each}
                    </select>
                {:else}
                    <select bind:value={appState.audioCodec}>
                        {#each AudioExtensions as format}
                            <option value={format}>{format}</option>
                        {/each}
                    </select>
                {/if}
            </div>
            {#if appState.convertType == "Video"}
                <div class="video-options">
                    <div class="option-label">{t("frameSize")}</div>
                    <div class="option-area">
                        <RadioGroup
                            options={["SizeNone", "360p", "480p", "720p", "1080p"]}
                            labels={["None", "360p", "480p", "720p", "1080p"]}
                            name="framesize"
                            checkedOption={appState.frameSize}
                            bind:group={appState.frameSize}
                        />
                    </div>
                    <div class="option-label">{t("videoRotation")}</div>
                    <div class="option-area">
                        <RadioGroup
                            options={["RotationNone", "90Clockwise", "90CounterClockwise"]}
                            labels={["None", "+90", "-90"]}
                            name="rotation"
                            checkedOption={appState.rotation}
                            bind:group={appState.rotation}
                        />
                    </div>
                </div>
            {/if}
            <div class="audio-options">
                <div class="option-label">{t("audioBitrate")}</div>
                <div class="option-area">
                    <RadioGroup
                        options={["BitrateNone", "128", "160", "192", "320"]}
                        labels={["None", "128", "160", "192", "320"]}
                        name="audioBitrate"
                        checkedOption={appState.audioBitrate}
                        bind:group={appState.audioBitrate}
                    />
                </div>
                <div class="option-label">{t("volume")}<label><input type="checkbox" class="max-volume" bind:checked={appState.maxVolume} />{t("maximizeVolue")}</label></div>
                <div class="option-area">
                    <input type="range" min="1" max="5" step="0.5" bind:value={appState.audioVolume} disabled={appState.maxVolume} />
                    <span id="volumeLabel">{`${parseFloat(appState.audioVolume) * 100}%`}</span>
                </div>
            </div>

            <div class="button">
                <button disabled={appState.converting} onclick={requestConvert}>{t("start")}</button>
                <button disabled={!appState.converting} onclick={requestCancelConvert}>{t("cancel")}</button>
                <button onclick={closeDialog}>{t("close")}</button>
            </div>
        </div>
    </div>
</div>

<style>
    select {
        position: relative;
        height: 30px;
        flex: 1;
        line-height: 30px;
        font-size: 16px;
        background-color: var(--input-bgcolor);
        color: var(--input-color);
        font-family: var(--font);
        text-indent: 5px;
    }
    select:focus {
        outline: none;
    }
</style>
