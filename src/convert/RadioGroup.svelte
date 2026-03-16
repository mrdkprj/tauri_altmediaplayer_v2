<script lang="ts" generics="T extends string | number">
    type DisableIf = {
        condition: boolean;
        target: T;
    };

    let {
        options,
        labels,
        name,
        checkedOption = null,
        disableIf = null,
        group = $bindable(),
    }: {
        options: T[];
        labels: string[];
        name: string;
        checkedOption: T | null;
        disableIf?: DisableIf | null;
        group: string;
    } = $props();
</script>

{#each options as option, index}
    <div class="radio">
        <input type="radio" value={option} bind:group {name} checked={option == checkedOption} disabled={disableIf ? disableIf.condition && option == disableIf.target : false} />
        <label for="">{labels[index]}</label>
    </div>
{/each}
