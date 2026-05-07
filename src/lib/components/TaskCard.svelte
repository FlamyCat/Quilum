<script lang="ts">
    import BaseCard from "./BaseCard.svelte";

    type Props = {
        title: string;
        description?: string;
        startTime: Date | null;
        endTime: Date | null;
        completed?: boolean;
        onToggle?: (completed: boolean) => Promise<void>;
        showTime?: boolean;
        href?: string;
    };

    let {
        title,
        description,
        startTime,
        endTime,
        completed = false,
        onToggle,
        showTime = false,
        href,
    }: Props = $props();
</script>

<BaseCard
    {title}
    {description}
    {startTime}
    {endTime}
    showCheckbox={true}
    checked={completed}
    {onToggle}
    {showTime}
    {href}
    class="opacity-50 [&:not(.completed)]:opacity-100 in-[.completed]:opacity-50"
/>
