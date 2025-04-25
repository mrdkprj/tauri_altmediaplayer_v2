<script lang="ts">
    import Slider from "./Slider.svelte";
    import { Buttons, handleKeyEvent } from "../constants";
    import { appState, dispatch } from "./appStateReducer";

    let {
        onMouseEnter,
        onUpdateTime,
        onUpdateVolume,
        onUpdateAmpLevel,
        onClickPlay,
        onClickStop,
        onClickPrevious,
        onClickNext,
        onClickMute,
        t,
    }: {
        onMouseEnter: () => void;
        onUpdateTime: (progress: number) => void;
        onUpdateVolume: (progress: number) => void;
        onUpdateAmpLevel: (progress: number) => void;
        onClickPlay: () => void;
        onClickStop: () => void;
        onClickPrevious: (button: number) => void;
        onClickNext: (button: number) => void;
        onClickMute: () => void;
        t: (key: keyof Mp.Labels) => string;
    } = $props();

    const formatTime = (secondValue: number) => {
        const hours = Math.floor(secondValue / 3600)
            .toString()
            .padStart(2, "0");
        const minutes = Math.floor((secondValue % 3600) / 60)
            .toString()
            .padStart(2, "0");
        const seconds = Math.floor((secondValue % 3600) % 60)
            .toString()
            .padStart(2, "0");

        return `${hours}:${minutes}:${seconds}`;
    };

    const getTimeTrackHoverTime = (progress: number) => {
        const time = $appState.media.videoDuration * progress;

        if (time <= 0) return "";

        return formatTime(time);
    };

    const updatePreventAutohide = (prevent: boolean) => {
        dispatch({ type: "preventAutohide", value: prevent });
    };
</script>

<div class="footer" onmouseenter={onMouseEnter} role="button" tabindex="-1">
    <div class="footer-menu">
        <div class="footer-top">
            <div class="time-area" onmouseenter={() => updatePreventAutohide(true)} onmouseleave={() => updatePreventAutohide(false)} role="button" tabindex="-1">
                <Slider
                    sliderClass={["time"]}
                    trackValueClass={["current-time"]}
                    thumbType="dot"
                    onSlide={onUpdateTime}
                    value={$appState.media.currentTime}
                    valuePosition="left"
                    displayFormatter={formatTime}
                    onTooltip={getTimeTrackHoverTime}
                    max={$appState.media.videoDuration}
                    offSet={-4}
                />
                <div class="track-value duration">{formatTime($appState.media.videoDuration)}</div>
            </div>
            <div class="btn-area">
                <div
                    class="btn-large"
                    onclick={onClickPlay}
                    onkeydown={handleKeyEvent}
                    onmouseenter={() => updatePreventAutohide(true)}
                    onmouseleave={() => updatePreventAutohide(false)}
                    role="button"
                    tabindex="-1"
                >
                    {#if $appState.playing}
                        <svg xmlns="http://www.w3.org/2000/svg" class="pause" fill="currentColor" viewBox="0 0 16 16">
                            <path d="M5.5 3.5A1.5 1.5 0 0 1 7 5v6a1.5 1.5 0 0 1-3 0V5a1.5 1.5 0 0 1 1.5-1.5zm5 0A1.5 1.5 0 0 1 12 5v6a1.5 1.5 0 0 1-3 0V5a1.5 1.5 0 0 1 1.5-1.5z" />
                        </svg>
                    {:else}
                        <svg xmlns="http://www.w3.org/2000/svg" class="play" fill="currentColor" viewBox="0 0 16 16">
                            <path d="m11.596 8.697-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393z" />
                        </svg>
                    {/if}
                </div>
                <div class="btn" onclick={onClickStop} onkeydown={handleKeyEvent} role="button" tabindex="-1">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                        <path d="M5 3.5h6A1.5 1.5 0 0 1 12.5 5v6a1.5 1.5 0 0 1-1.5 1.5H5A1.5 1.5 0 0 1 3.5 11V5A1.5 1.5 0 0 1 5 3.5z" />
                    </svg>
                </div>
                <div
                    class="btn"
                    onclick={() => onClickPrevious(Buttons.left)}
                    oncontextmenu={() => onClickPrevious(Buttons.right)}
                    onkeydown={handleKeyEvent}
                    onmouseenter={() => updatePreventAutohide(true)}
                    onmouseleave={() => updatePreventAutohide(false)}
                    role="button"
                    tabindex="-1"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                        <path
                            d="M.5 3.5A.5.5 0 0 0 0 4v8a.5.5 0 0 0 1 0V8.753l6.267 3.636c.54.313 1.233-.066 1.233-.697v-2.94l6.267 3.636c.54.314 1.233-.065 1.233-.696V4.308c0-.63-.693-1.01-1.233-.696L8.5 7.248v-2.94c0-.63-.692-1.01-1.233-.696L1 7.248V4a.5.5 0 0 0-.5-.5z"
                        />
                    </svg>
                </div>
                <div
                    class="btn"
                    onclick={() => onClickNext(Buttons.left)}
                    oncontextmenu={() => onClickNext(Buttons.right)}
                    onkeydown={handleKeyEvent}
                    onmouseenter={() => updatePreventAutohide(true)}
                    onmouseleave={() => updatePreventAutohide(false)}
                    role="button"
                    tabindex="-1"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                        <path
                            d="M15.5 3.5a.5.5 0 0 1 .5.5v8a.5.5 0 0 1-1 0V8.753l-6.267 3.636c-.54.313-1.233-.066-1.233-.697v-2.94l-6.267 3.636C.693 12.703 0 12.324 0 11.693V4.308c0-.63.693-1.01 1.233-.696L7.5 7.248v-2.94c0-.63.693-1.01 1.233-.696L15 7.248V4a.5.5 0 0 1 .5-.5z"
                        />
                    </svg>
                </div>
                <div class="btn-separator"></div>
                {#if $appState.converting}
                    <div class="convert-state">
                        <div class="ring">
                            <div class="lds-dual-ring"></div>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
        <div class="footer-bottom">
            <div class="volume-area" onmouseenter={() => updatePreventAutohide(true)} onmouseleave={() => updatePreventAutohide(false)} role="button" tabindex="-1">
                <Slider sliderClass={["volume"]} thumbType="dot" onSlide={onUpdateVolume} value={$appState.media.videoVolume} valuePosition="right" />
            </div>
            <div class="amp-area" class:mute={$appState.media.mute} onmouseenter={() => updatePreventAutohide(true)} onmouseleave={() => updatePreventAutohide(false)} role="button" tabindex="-1">
                <div class="btn sound" onclick={onClickMute} onkeydown={handleKeyEvent} role="button" tabindex="-1" title={t("mute")}>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 16 16">
                        <path
                            d="M9 4a.5.5 0 0 0-.812-.39L5.825 5.5H3.5A.5.5 0 0 0 3 6v4a.5.5 0 0 0 .5.5h2.325l2.363 1.89A.5.5 0 0 0 9 12V4zm3.025 4a4.486 4.486 0 0 1-1.318 3.182L10 10.475A3.489 3.489 0 0 0 11.025 8 3.49 3.49 0 0 0 10 5.525l.707-.707A4.486 4.486 0 0 1 12.025 8z"
                        />
                    </svg>
                </div>
                <Slider sliderClass={["amp"]} thumbType="lever" onSlide={onUpdateAmpLevel} value={$appState.media.ampLevel} valuePosition="right" />
            </div>
        </div>
    </div>
</div>
