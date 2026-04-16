<script lang="ts">
    import type { Component } from "svelte";
    import type { LucideProps } from "@lucide/svelte";

    type Props = {
        icon: Component<LucideProps>;
        label: string;
        href: string;
        active?: boolean;
        color?: string;
        size?: number;
        strokeWidth?: number;
        ariaLabel: string;
    };

    let {
        icon,
        label,
        href,
        active = false,
        color = "currentColor",
        size = 24,
        strokeWidth = 2,
        ariaLabel,
    }: Props = $props();

    const Icon = icon;
</script>

<a
    {href}
    aria-label="{ariaLabel}"
    class="flex items-center gap-2 transition-colors duration-200 {active
        ? 'text-black dark:text-white'
        : 'text-gray-500 dark:text-gray-400'}"
>
    <Icon {color} {size} {strokeWidth} />
    <span class="text-sm font-medium">{label}</span>
</a>
