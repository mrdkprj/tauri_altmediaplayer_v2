<script lang="ts" generics="T extends string | number">

    type DisableIf = {
        condition:boolean;
        target:T;
    }

    export let options:T[];
    export let labels:string[];
    export let name:string;
    export let checkedOption:T | null = null;
    export let disableIf:DisableIf | null = null
    export let onChange:(e:Mp.RadioGroupChangeEvent<T>) => void;

    const _onChange = (e:Event) => {
        const target = e.target as HTMLInputElement
        onChange({value:target.value as T})
    }

</script>

{#each options as option, index}
    <div class="radio">
        <input type="radio"
            value={option}
            name={name}
            checked={option == checkedOption}
            on:change={_onChange}
            disabled={disableIf ? disableIf.condition && option == disableIf.target : false}
        />
        <label for="">{labels[index]}</label>
    </div>
{/each}
