<script lang="ts">
    let { color = $bindable<string>("#000") } = $props();
    let colorPicker = $state<HTMLElement>();

    $effect(() => {
        colorPicker!.style.backgroundColor = color!;
    })
</script>

<div bind:this={colorPicker} class="size-10 rounded-lg">
    <input placeholder="{color}" type="color" bind:value={color} class="opacity-0 size-full">
</div>
