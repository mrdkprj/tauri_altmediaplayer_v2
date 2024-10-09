<script lang="ts">
    import { onMount } from "svelte";

    export let sliderClass: string[];
    export let trackValueClass: string[] = [];
    export let thumbType: "dot" | "lever";
    export let onSlide: (progress: number) => void;
    export let displayFormatter: ((progress: number) => string) | null = null;
    export let onTooltip: ((progress: number) => string) | null = null;
    export let max: number | null = null;
    export let value: number;
    export let valuePosition: "left" | "right";
    export let offSet: number | null = null;

    type TooltipState = {
        visible: boolean;
        text: string;
        top: number;
        left: number;
    };

    const THUM_WIDTH = 8;

    let sliding = false;
    let rect: DOMRect;
    let toolTip: TooltipState = { visible: false, text: "", top: 0, left: 0 };
    let slider: HTMLDivElement;
    let startX = 0;

    const startSlide = (e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        sliding = true;
        startX = e.clientX;
    };

    const moveSlider = (e: MouseEvent) => {
        if (!sliding || e.clientX == startX) return;

        const rect = slider.getBoundingClientRect();
        const progress = (e.clientX - rect.left) / rect.width;

        if (progress > 1 || progress < 0) return;

        onSlide(progress);
    };

    const endSlide = (e: MouseEvent) => {
        if (sliding) {
            e.preventDefault();
            e.stopPropagation();
            sliding = false;
        }
    };

    const onTrackMousedown = (e: MouseEvent) => {
        const offset = offSet ? offSet : 0;
        const progress = (e.offsetX - offset) / rect.width;

        onSlide(progress);
    };

    const showTooltip = (e: MouseEvent) => {
        if (!onTooltip) return;

        const progress = (e.clientX - rect.left) / rect.width;
        const text = onTooltip(progress);

        if (!text) return hideTooltip();

        toolTip = { visible: true, text, top: rect.bottom + 10, left: e.clientX + 15 };
    };

    const hideTooltip = () => {
        if (!onTooltip) return;

        toolTip = { ...toolTip, visible: false, text: "" };
    };

    $: getRate = () => {
        if (max) {
            return `${(value / max) * 100}%`;
        }

        return `${Math.floor(value * 100)}%`;
    };

    const setRect = () => {
        rect = slider.getBoundingClientRect();
    };

    onMount(() => {
        setRect();
    });
</script>

<svelte:window on:resize={setRect} />
<svelte:document on:mousemove={moveSlider} on:mouseup={endSlide} />

{#if toolTip.visible}
    <div class="tooltip" style="left:{toolTip.left}px; top:{toolTip.top}px">{toolTip.text}</div>
{/if}

{#if valuePosition === "left"}
    <div class="track-value {trackValueClass?.join(' ')}">{displayFormatter ? displayFormatter(value) : getRate()}</div>
{/if}

<div
    class="slider {sliderClass.join(' ')}"
    class:sliding
    bind:this={slider}
    on:mousedown={onTrackMousedown}
    on:mouseenter={showTooltip}
    on:mousemove={showTooltip}
    on:mouseleave={hideTooltip}
    role="button"
    tabindex="-1"
>
    <div class="track" style:width={getRate()}></div>
    <div
        class="thumb"
        class:lever={thumbType === "lever"}
        style="left:max({getRate()} - {THUM_WIDTH}px, 0px)"
        on:mousedown={startSlide}
        title={onTooltip ? "" : getRate()}
        role="button"
        tabindex="-1"
    ></div>
</div>

{#if valuePosition === "right"}
    <div class="track-value {trackValueClass?.join(' ')}">{displayFormatter ? displayFormatter(value) : getRate()}</div>
{/if}
