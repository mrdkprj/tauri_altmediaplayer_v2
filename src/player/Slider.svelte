<script lang="ts">
    import { onMount } from "svelte";

    let {
        sliderClass,
        trackValueClass = [],
        thumbType,
        onSlide,
        displayFormatter = null,
        onTooltip = null,
        max = null,
        value,
        valuePosition,
        offSet = null,
    }: {
        sliderClass: string[];
        trackValueClass?: string[];
        thumbType: "dot" | "lever";
        onSlide: (progress: number) => void;
        displayFormatter?: ((progress: number) => string) | null;
        onTooltip?: ((progress: number) => string) | null;
        max?: number | null;
        value: number;
        valuePosition: "left" | "right";
        offSet?: number | null;
    } = $props();

    type TooltipState = {
        visible: boolean;
        text: string;
        top: number;
        left: number;
    };
    let sliding = $state(false);
    let toolTip = $state<TooltipState>({ visible: false, text: "", top: 0, left: 0 });

    const THUM_WIDTH = 8;

    let rect: DOMRect;
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

    const rate = $derived.by(() => {
        if (max) {
            return `${(value / max) * 100}%`;
        }

        return `${Math.floor(value * 100)}%`;
    });

    const setRect = () => {
        rect = slider.getBoundingClientRect();
    };

    onMount(() => {
        setRect();
    });
</script>

<svelte:window onresize={setRect} />
<svelte:document onmousemove={moveSlider} onmouseup={endSlide} />

{#if toolTip.visible}
    <div class="tooltip" style="left:{toolTip.left}px; top:{toolTip.top}px">{toolTip.text}</div>
{/if}

{#if valuePosition === "left"}
    <div class="track-value {trackValueClass?.join(' ')}">{displayFormatter ? displayFormatter(value) : rate}</div>
{/if}

<div
    class="slider {sliderClass.join(' ')}"
    class:sliding
    bind:this={slider}
    onmousedown={onTrackMousedown}
    onmouseenter={showTooltip}
    onmousemove={showTooltip}
    onmouseleave={hideTooltip}
    role="button"
    tabindex="-1"
>
    <div class="track" style:width={rate}></div>
    <div class="thumb" class:lever={thumbType === "lever"} style="left:max({rate} - {THUM_WIDTH}px, 0px)" onmousedown={startSlide} title={onTooltip ? "" : rate} role="button" tabindex="-1"></div>
</div>

{#if valuePosition === "right"}
    <div class="track-value {trackValueClass?.join(' ')}">{displayFormatter ? displayFormatter(value) : rate}</div>
{/if}
