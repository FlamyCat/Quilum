<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        title: string;
        description?: string;
        startTime: Date | null;
        endTime: Date | null;
        showCheckbox?: boolean;
        checked?: boolean;
        onToggle?: (checked: boolean) => Promise<void>;
        children?: Snippet;
        class?: string;
        showTime?: boolean;
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
        showTime = false,
    }: Props = $props();

    function formatTime(date: Date | null): string {
        if (date === null) return "-";
        return date.toLocaleTimeString("ru-RU", {
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
        <h3 class="font-semibold text-black dark:text-white truncate">
            <span class="strikethrough">{title}</span>
        </h3>
        {#if description}
            <p class="text-gray-500 dark:text-gray-400 truncate">
                <span class="strikethrough">{description}</span>
            </p>
        {/if}
        {#if children}
            {@render children()}
        {/if}
    </div>
{/snippet}

{#snippet time()}
    {#if showTime}
        <div
            class="font-light flex flex-col h-full justify-evenly text-gray-500 dark:text-gray-400"
        >
            <span>{formatTime(startTime)}</span>
            <span>{formatTime(endTime)}</span>
        </div>
    {/if}
{/snippet}

<div
    class="h-20 transition duration-300 flex flex-row gap-4 items-center bg-white dark:bg-slate-700 rounded-lg px-4 shadow-md {containerClass}"
>
    {@render checkbox()}
    {@render time()}
    {@render titleAndDescription()}
</div>

<style>
    .completed .strikethrough {
        text-decoration: line-through;
        text-decoration-color: currentColor;
        text-decoration-thickness: 1px;
    }
</style>
