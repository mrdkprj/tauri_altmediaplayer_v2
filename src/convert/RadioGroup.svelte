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
        onChange,
    }: {
        options: T[];
        labels: string[];
        name: string;
        checkedOption: T | null;
        disableIf?: DisableIf | null;
        onChange: (e: Mp.RadioGroupChangeEvent<T>) => void;
    } = $props();

    const _onChange = (e: Event) => {
        const target = e.target as HTMLInputElement;
        onChange({ value: target.value as T });
    };
</script>

{#each options as option, index}
    <div class="radio">
        <input type="radio" value={option} {name} checked={option == checkedOption} onchange={_onChange} disabled={disableIf ? disableIf.condition && option == disableIf.target : false} />
        <label for="">{labels[index]}</label>
    </div>
{/each}
