<script lang="ts">
    import type { Snippet } from "svelte";
    import { fly } from "svelte/transition";

    type Props = {
        title: string;
        description?: string;
        startTime: Date;
        endTime: Date;
        showCheckbox?: boolean;
        checked?: boolean;
        onToggle?: (checked: boolean) => Promise<void>;
        children?: Snippet;
        class?: string;
    };

    let {
        title,
        description,
        startTime,
        endTime,
        showCheckbox = false,
        checked = false,
        onToggle,
        children,
        class: className = "",
    }: Props = $props();

    function formatTime(date: Date): string {
        return date.toLocaleTimeString("en-GB", {
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    async function handleToggle(event: Event) {
        const target = event.target as HTMLInputElement;
        await onToggle?.(target.checked);
    }

    let containerClass = $derived(
        checked ? `completed ${className}` : className,
    );
</script>

{#snippet checkbox()}
    <div class="w-4 flex items-center justify-center">
        {#if showCheckbox}
            <input
                type="checkbox"
                class="w-5 h-5 rounded border-gray-300 cursor-pointer accent-black dark:accent-white"
                {checked}
                onchange={handleToggle}
            />
        {/if}
    </div>
{/snippet}

{#snippet titleAndDescription()}
    <div class="h-full flex-1 flex flex-col min-w-0 justify-evenly">
        <h3 class="font-semibold text-black dark:text-white">
            <span class="strikethrough">{title}</span>
        </h3>
        <p class="text-gray-500 dark:text-gray-400 truncate">
            <span class="strikethrough">{description}</span>
        </p>
        {#if children}
            {@render children()}
        {/if}
    </div>
{/snippet}

{#snippet time()}
    <div
        class="font-light flex flex-col h-full justify-evenly text-gray-500 dark:text-gray-400"
    >
        <span>{formatTime(startTime)}</span>
        <span>{formatTime(endTime)}</span>
    </div>
{/snippet}

<div
    class="h-20 transition duration-300 flex flex-row gap-4 items-center bg-white dark:bg-slate-800 rounded-lg px-4 shadow-md {containerClass}"
>
    {@render checkbox()}
    {@render time()}
    {@render titleAndDescription()}
</div>

<style>
    .strikethrough {
        position: relative;
        display: inline-block;
    }

    .strikethrough::after {
        content: "";
        position: absolute;
        left: 0;
        top: 50%;
        height: 1px;
        width: 0;
        background: currentColor;
        transition: width 0.5s cubic-bezier(0.33, 1, 0.68, 1);
    }

    .completed .strikethrough::after {
        width: 100%;
    }
</style>
